use anyhow::Result;
use log::debug;
use serenity::builder::{CreateCommand, CreateCommandOption, CreateEmbed};
use serenity::model::application::{CommandInteraction, CommandOptionType};
use serenity::prelude::Context;
use tyche::dice::roller::FastRand;
use tyche::expr::Describe;
use tyche::Expr;

use crate::commands::{BotCommand, CommandStatus, InteractionUtil, SendEmbed};

/// A maximum number of dices that can be rolled at once.
const MAX_DICE_NUM: usize = 30;

/// A command to roll dices.
pub struct RollCommand;

#[naming]
#[serenity::async_trait]
impl BotCommand for RollCommand {
    fn create(&self) -> CreateCommand {
        CreateCommand::new(self.name())
            .description("지정된 주사위를 굴립니다.")
            .add_option(
                CreateCommandOption::new(
                    CommandOptionType::String,
                    "dice",
                    "굴릴 주사위 식 (예: 3d4 + 1)",
                )
                .required(true),
            )
            .add_option(
                CreateCommandOption::new(
                    CommandOptionType::String,
                    "comment",
                    "주사위 굴림에 대한 설명",
                )
            )
    }

    async fn execute(
        &self,
        ctx: &Context,
        interaction: &CommandInteraction,
    ) -> Result<CommandStatus> {
        let dice = interaction.get_string_option("dice".into()).unwrap();

        let comment = interaction
            .get_string_option("comment".into())
            .map(|comment| format!(": {}", comment))
            .unwrap_or_default();

        match RollCommand::evaluate_dice_expr(dice) {
            Ok((evaluated, description)) => {
                interaction
                    .send_embed(
                        ctx,
                        CreateEmbed::new()
                            .title(format!(
                                "{}의 주사위 굴림{}",
                                interaction.get_nickname(),
                                comment
                            ))
                            .field(format!(":game_die: {}", evaluated), description, false),
                    )
                    .await?;

                Ok(CommandStatus::Ok)
            }
            Err(message) => {
                debug!(
                    "Failed to evaluate dice expression: {} (reason: {})",
                    dice, message
                );

                Ok(CommandStatus::Err(message))
            }
        }
    }
}

impl RollCommand {
    /// Evaluates a dice expression.
    pub fn evaluate_dice_expr(expr: &str) -> Result<(i32, String), String> {
        // Parse the expression.
        let expr: Expr = expr
            .parse()
            .map_err(|err: tyche::parse::Error| err.to_string())?;

        // Evaluate the expression.
        let mut roller = FastRand::default();
        let result = expr.eval(&mut roller).map_err(|err| err.to_string())?;

        // Calculate the result.
        let evaluated = result.calc().map_err(|err| err.to_string())?;

        Ok((evaluated, result.describe(Some(MAX_DICE_NUM))))
    }
}
