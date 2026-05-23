extern crate cmd_macro;

use anyhow::Result;
use log::info;
use serenity::prelude::GatewayIntents;
use serenity::Client;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::commands::channel::{load_channels, ChannelStore};
use crate::commands::create_sheet::{load_sheets, SheetStore};
use crate::config::BotConfig;
use crate::handler::BotHandler;
use crate::logging::Logger;

/// Initializes a bot and lets the bot start.
async fn start_bot() -> Result<()> {
    // Read the configurations.
    let config = BotConfig::get();

    // Build a client.
    let intents = GatewayIntents::non_privileged() | GatewayIntents::MESSAGE_CONTENT;
    let mut client = Client::builder(&config.discord_token, intents)
        .event_handler(BotHandler)
        .await?;

    // Initialize the in-memory sheet store.
    {
        let mut data = client.data.write().await;
        let sheets = load_sheets();
        data.insert::<SheetStore>(Arc::new(RwLock::new(sheets)));
        
        let channels = load_channels();
        data.insert::<ChannelStore>(Arc::new(RwLock::new(channels)));
    }

    // Launch the client.
    client.start().await?;

    Ok(())
}

async fn start_process() -> Result<()> {
    // Initialize the file logging.
    Logger::init_file_logging().await?;

    info!(
        "----------------------\n  cthulhu bot v{}\n----------------------",
        env!("CARGO_PKG_VERSION")
    );

    Logger::enable_daily_reports();

    start_bot().await?;

    Ok(())
}

#[tokio::main]
async fn main() {
    Logger::init();

    let result = start_process().await;
    Logger::log_err(&result).await;
}

pub mod commands;
pub mod config;
pub mod handler;
pub mod logging;
pub mod webhook;
