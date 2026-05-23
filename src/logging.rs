use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::Write;

use anyhow::Result;
use chrono::{Timelike, Utc};
use log::{error, info, Log};
use once_cell::sync::{Lazy, OnceCell};
use serenity::prelude::Mutex;
use tokio::sync::mpsc::{self, Receiver, Sender};
use tokio::time::Duration;

use crate::config::{BotConfig, BotConfigError};

/// A handler for the log.
pub struct Logger;

static LOGGER: Logger = Logger;

const LOG_BUFFER_SIZE: usize = 0x1000;

static LOG_SENDER: OnceCell<Sender<String>> = OnceCell::new();

impl Logger {
    pub fn init() {
        if let Err(_) = log::set_logger(&LOGGER) {
            panic!("Failed to set the logger.");
        }
        log::set_max_level(log::LevelFilter::Info);
    }

    pub async fn init_file_logging() -> Result<()> {
        // Open the log file to check if it is writable.
        let log_file = Logger::open_log_file().await?;

        // Create a channel for file logging.
        let (tx, rx) = mpsc::channel::<String>(LOG_BUFFER_SIZE);

        // Spawn a task for file logging.
        tokio::spawn(async move {
            Logger::file_logging_loop(log_file, rx).await;
        });

        // Initialize the log sender.
        if let Err(_) = LOG_SENDER.set(tx) {
            panic!("Re-initialized the log sender.");
        }

        Ok(())
    }

    async fn file_logging_loop(mut file: File, mut rx: Receiver<String>) -> ! {
        loop {
            while let Some(text) = rx.recv().await {
                if let Err(err) = file.write_all(text.as_bytes()) {
                    print!("{}", text);

                    panic!("Failed to write the log to the file. (Info: {})", err);
                }
            }
        }
    }

    async fn open_log_file() -> Result<File> {
        let config = BotConfig::get();

        let file = OpenOptions::new()
            .append(true)
            .create(true)
            .open(&config.log_path)
            .map_err(|_| {
                BotConfigError::new(&format!(
                    "Cannot open or create the log file \"{}\".",
                    config.log_path
                ))
            })?;

        Ok(file)
    }
}

impl Log for Logger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        // Only track the logs from this bot.
        metadata.target().starts_with(env!("CARGO_PKG_NAME"))
    }

    fn log(&self, record: &log::Record) {
        let date = Utc::now().to_rfc3339();
        let text = record
            .args()
            .to_string()
            .lines()
            .map(|line| format!("{:35} [{:5}] {}\n", date, record.level(), line))
            .collect::<Vec<_>>()
            .concat();

        let sender = LOG_SENDER.get();
        match sender {
            Some(sender) => {
                // Try to send the log to the file logging task.
                if let Err(err) = sender.try_send(text.clone()) {
                    // If failed, print the log to stderr and panic.
                    eprint!("{}", text);

                    panic!(
                        "Failed to send the log to the file logging task. (Info: {})",
                        err
                    );
                }
            }
            None => {
                // If the sender is not initialized, print the log to the stdout or stderr.
                if record.level() <= log::Level::Warn {
                    eprint!("{}", text);
                } else {
                    print!("{}", text);
                }
            }
        }
    }

    fn flush(&self) {}
}

impl Logger {
    /// Emits error logs.
    pub async fn log_err(result: &Result<()>) {
        if let Err(err) = result {
            let text = err.to_string()
                + "\n"
                + &err
                    .chain()
                    .skip(1)
                    .map(|line| format!("  because: {}\n", line))
                    .collect::<Vec<_>>()
                    .concat();
            
            // 프로그램이 즉시 종료되어 비동기 로거가 묻히는 현상을 방지
            eprintln!("\n[치명적 오류 발생 - 봇 종료됨]\n{}\n", text);
            error!("{}", text);
            // 파일 로거가 기록을 마칠 수 있도록 잠시 대기
            tokio::time::sleep(Duration::from_millis(500)).await;
        }
    }

    /// Publishes a report of the events everyday.
    pub fn enable_daily_reports() {
        tokio::spawn(async {
            info!("The daily report system is now enabled.");

            loop {
                // Wait at least 1 hour.
                tokio::time::sleep(Duration::from_secs(60 * 60)).await;

                // Calculate the waiting duration until the next report time.
                let waiting_duration =
                    Logger::duration_to_next_report().unwrap_or(Duration::from_secs(60 * 60 * 23));
                tokio::time::sleep(waiting_duration).await;

                BotEventCounter::report_to_log().await;
            }
        });
    }

    /// Calculates the duration until the next report time (00:00:00 UTC).
    fn duration_to_next_report() -> Option<Duration> {
        let now = Utc::now();
        let delta = (now + chrono::Duration::days(1))
            .naive_utc()
            .with_hour(0)?
            .with_minute(0)?
            .with_second(0)?
            .with_nanosecond(0)?
            - now.naive_utc();
        delta.to_std().ok()
    }
}

/// Counters of the events.
static EVENT_COUNTERS: Lazy<Mutex<HashMap<String, u32>>> = Lazy::new(|| Mutex::new(HashMap::new()));

pub struct BotEventCounter;

impl BotEventCounter {
    pub async fn increment(name: &str) {
        let mut counter = EVENT_COUNTERS.lock().await;
        let command_counter = counter.get_mut(name);
        if let Some(counter) = command_counter {
            *counter += 1;
        } else {
            counter.insert(name.to_string(), 1);
        }
    }

    /// Reports the counters to the log and **resets** the counters.
    pub async fn report_to_log() {
        let mut counter = EVENT_COUNTERS.lock().await;

        let report = if counter.len() == 0 {
            "  Nothing".to_string()
        } else {
            counter
                .iter()
                .map(|(key, value)| format!("  {}: {}\n", key, value))
                .collect::<Vec<_>>()
                .concat()
        };

        // Log the report at once to avoid interleaving with other logs.
        info!("Usage report\n{}", report);

        *counter = HashMap::new();

        // Reset the counters immediately
        // to prevent the counters being incremented after reporting.
        info!("Reset the event counters.");
    }
}
