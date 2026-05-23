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

        // 3초 타임아웃 방지 및 응답 메시지 숨기기를 위한 지연(Defer) 처리
        interaction.create_response(ctx, CreateInteractionResponse::Defer(CreateInteractionResponseMessage::new().ephemeral(true))).await?;

        // 이름 없는 웹훅 전송 시도 (디스코드 정책으로 빈 이름이 막힐 경우 "내레이션"으로 대체)
        if send_webhook(ctx, interaction.channel_id, "\u{200B}\u{200B}", None, &format!("***{}***", content)).await.is_err() {
            send_webhook(ctx, interaction.channel_id, "내레이션", None, &format!("***{}***", content)).await?;
        }

        // 지연(Defer)했던 응답 메시지를 삭제하여 깔끔하게 만듦
        interaction.delete_response(ctx).await?;

        Ok(CommandStatus::Ok)
    }
}