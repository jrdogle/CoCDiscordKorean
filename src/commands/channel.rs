use std::collections::HashSet;
use std::sync::Arc;

use anyhow::Result;
use serenity::builder::{CreateCommand, CreateEmbed};
use serenity::model::application::CommandInteraction;
use serenity::prelude::{Context, TypeMapKey};
use tokio::sync::RwLock;

use crate::commands::{BotCommand, CommandStatus, SendEmbed};

pub struct ChannelStore;
impl TypeMapKey for ChannelStore {
    type Value = Arc<RwLock<HashSet<u64>>>;
}

pub fn load_channels() -> HashSet<u64> {
    let file_path = std::path::Path::new("channels.json");
    if let Ok(json) = std::fs::read_to_string(file_path) {
        if let Ok(channels) = serde_json::from_str::<Vec<u64>>(&json) {
            return channels.into_iter().collect();
        }
    }
    HashSet::new()
}

pub async fn save_channels(channels: &HashSet<u64>) {
    let vec: Vec<u64> = channels.iter().cloned().collect();
    if let Ok(json) = serde_json::to_string_pretty(&vec) {
        let file_path = std::path::Path::new("channels.json");
        let _ = tokio::fs::write(file_path, json).await;
    }
}

pub struct BotJoinCommand;

#[serenity::async_trait]
impl BotCommand for BotJoinCommand {
    fn name(&self) -> &str {
        "봇입장"
    }

    fn create(&self) -> CreateCommand {
        CreateCommand::new(self.name()).description("이 채널에서 봇이 작동하도록 활성화합니다.")
    }

    async fn execute(&self, ctx: &Context, interaction: &CommandInteraction) -> Result<CommandStatus> {
        let channel_id = interaction.channel_id.get();
        let store = {
            let data = ctx.data.read().await;
            data.get::<ChannelStore>().cloned()
        };

        if let Some(store) = store {
            let mut channels = store.write().await;
            channels.insert(channel_id);
            save_channels(&channels).await;
        }

        let embed = CreateEmbed::new()
            .title("봇 활성화")
            .description("✅ 이 채널에서 봇이 작동합니다.");
        interaction.send_embed(ctx, embed).await?;

        Ok(CommandStatus::Ok)
    }
}

pub struct BotLeaveCommand;

#[serenity::async_trait]
impl BotCommand for BotLeaveCommand {
    fn name(&self) -> &str {
        "봇퇴장"
    }

    fn create(&self) -> CreateCommand {
        CreateCommand::new(self.name()).description("이 채널에서 봇이 작동하지 않도록 비활성화합니다.")
    }

    async fn execute(&self, ctx: &Context, interaction: &CommandInteraction) -> Result<CommandStatus> {
        let channel_id = interaction.channel_id.get();
        let store = {
            let data = ctx.data.read().await;
            data.get::<ChannelStore>().cloned()
        };

        if let Some(store) = store {
            let mut channels = store.write().await;
            channels.remove(&channel_id);
            save_channels(&channels).await;
        }

        let embed = CreateEmbed::new()
            .title("봇 비활성화")
            .description("💤 이 채널에서 더 이상 봇이 작동하지 않습니다.");
        interaction.send_embed(ctx, embed).await?;

        Ok(CommandStatus::Ok)
    }
}