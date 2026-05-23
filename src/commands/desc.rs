use anyhow::Result;
use serenity::builder::{CreateCommand, CreateCommandOption, CreateInteractionResponse, CreateInteractionResponseMessage};
use serenity::model::application::{CommandInteraction, CommandOptionType};
use serenity::prelude::Context;

use crate::commands::{BotCommand, CommandStatus, InteractionUtil};
use crate::webhook::send_webhook;

/// 내레이션 출력 명령어
pub struct DescCommand;

#[serenity::async_trait]
impl BotCommand for DescCommand {
    fn name(&self) -> &str {
        "내레이션"
    }

    fn create(&self) -> CreateCommand {
        CreateCommand::new(self.name())
            .description("캐릭터 이름 없이 내레이션을 출력합니다.")
            .add_option(CreateCommandOption::new(CommandOptionType::String, "내용", "출력할 내레이션 내용").required(true))
    }

    async fn execute(&self, ctx: &Context, interaction: &CommandInteraction) -> Result<CommandStatus> {
        let content = interaction.get_string_option("내용".into()).unwrap();

        // \u{200B} (Zero-width space)를 사용하여 보이지 않는 이름으로 처리
        send_webhook(ctx, interaction.channel_id, "\u{200B}", None, &format!("***{}***", content)).await?;

        interaction.create_response(ctx, CreateInteractionResponse::Message(CreateInteractionResponseMessage::new().content("내레이션을 전송했습니다.").ephemeral(true))).await?;
        Ok(CommandStatus::Ok)
    }
}