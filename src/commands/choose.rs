use anyhow::Result;
use rand::Rng;
use serenity::builder::{CreateCommand, CreateCommandOption, CreateEmbed};
use serenity::model::application::{CommandInteraction, CommandOptionType};
use serenity::prelude::Context;

use crate::commands::{BotCommand, CommandStatus, InteractionUtil, SendEmbed};

/// A command that make a random choice.
pub struct ChooseCommand;

#[serenity::async_trait]
impl BotCommand for ChooseCommand {
    fn name(&self) -> &str {
        "선택"
    }

    fn create(&self) -> CreateCommand {
        CreateCommand::new(self.name())
            .description("주어진 항목 중 하나를 무작위로 선택합니다.")
            .add_option(
                CreateCommandOption::new(
                    CommandOptionType::String,
                    "선택지",
                    "쉼표로 구분된 선택지 (예: 사과,바나나,포도)",
                )
                .required(true),
            )
    }

    async fn execute(
        &self,
        ctx: &Context,
        interaction: &CommandInteraction,
    ) -> Result<CommandStatus> {
        let choices: Vec<&str> = interaction
            .get_string_option("선택지".into())
            .unwrap()
            .split(",")
            .collect();

        let author = interaction.get_nickname();

        let selected = rand::thread_rng().gen_range(0..choices.len());

        interaction
            .send_embed(
                ctx,
                CreateEmbed::new()
                    .title(format!("{}의 선택", author))
                    .field(
                        format!("**{}**", choices[selected]),
                        format!("선택지: {}", choices.join(", ")),
                        false,
                    ),
            )
            .await?;

        Ok(CommandStatus::Ok)
    }
}
