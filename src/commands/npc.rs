use anyhow::Result;
use serenity::builder::{CreateCommand, CreateCommandOption, CreateInteractionResponse, CreateInteractionResponseMessage};
use serenity::model::application::{CommandInteraction, CommandOptionType};
use serenity::prelude::Context;

use crate::commands::{BotCommand, CommandStatus, InteractionUtil};
use crate::webhook::send_webhook;

/// NPC 대사 출력 명령어
pub struct NpcCommand;

#[serenity::async_trait]
impl BotCommand for NpcCommand {
    fn name(&self) -> &str {
        "대사출력"
    }

    fn create(&self) -> CreateCommand {
        CreateCommand::new(self.name())
            .description("NPC의 이름으로 대사를 출력합니다.")
            .add_option(CreateCommandOption::new(CommandOptionType::String, "이름", "NPC 이름").required(true))
            .add_option(CreateCommandOption::new(CommandOptionType::String, "대사", "출력할 대사").required(true))
    }

    async fn execute(&self, ctx: &Context, interaction: &CommandInteraction) -> Result<CommandStatus> {
        let name = interaction.get_string_option("이름".into()).unwrap();
        let speech = interaction.get_string_option("대사".into()).unwrap();

        send_webhook(ctx, interaction.channel_id, name, None, speech).await?;

        interaction.create_response(ctx, CreateInteractionResponse::Message(CreateInteractionResponseMessage::new().content("NPC 대사를 전송했습니다.").ephemeral(true))).await?;

        Ok(CommandStatus::Ok)
    }
}