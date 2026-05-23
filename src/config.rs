use std::env;
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::fs::File;
use std::io::BufReader;
use std::sync::OnceLock;

use anyhow::Result;
use log::error;
use serde::Deserialize;

/// Holds error information related to the process of loading the configurations.
/// This should be used to handle errors triggered before the logging system is up.
#[derive(Debug)]
pub struct BotConfigError {
    message: String,
}

impl Display for BotConfigError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl Error for BotConfigError {}

impl BotConfigError {
    pub fn new(message: &str) -> Self {
        BotConfigError {
            message: message.to_string(),
        }
    }
}

/// A set of configurations of this bot.
#[derive(Deserialize)]
pub struct BotConfig {
    pub discord_token: String,
    pub log_path: String,
    pub status_message: String,
    pub database_url: Option<String>,
}

/// Holds the configurations of this bot. You need to call `BotConfig::load_from_file` before using this.
static BOT_CONFIG: OnceLock<BotConfig> = OnceLock::new();

impl BotConfig {
    /// Gets the configurations of this bot.
    pub fn get() -> &'static BotConfig {
        BOT_CONFIG.get_or_init(|| match BotConfig::load_from_file() {
            Ok(config) => config,
            Err(err) => {
                error!("Failed to load the config. (Info: {})", err);

                panic!("Failed to load the config.");
            }
        })
    }

    /// Loads `BOT_CONFIG` from `./config.json`.
    fn load_from_file() -> Result<BotConfig> {
        let config_file = std::path::Path::new("config.json").to_path_buf();
        if config_file.exists() {
            let config_file = File::open(config_file)?;
            let reader = BufReader::new(config_file);

            let config = serde_json::from_reader(reader)?;
            Ok(config)
        } else {
            Err(BotConfigError::new(
                "No configuration file is provided. Create a new \"config.json\" in the directory where this bot is placed.",
            ))?
        }
    }
}
