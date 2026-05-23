use anyhow::Result;
use rand::Rng;
use serenity::builder::{CreateCommand, CreateCommandOption, CreateEmbed};
use serenity::model::application::{CommandInteraction, CommandOptionType};
use serenity::prelude::Context;

use crate::commands::{BotCommand, CommandStatus, InteractionUtil, SendEmbed};

/// A helper for skill rolls.
struct SkillHelper;

/// A command that does a skill roll. It follows Call of Cthulhu 7th Edition.
pub struct Sk7Command;

/// A command that does a skill roll. It follows Delta Green.
pub struct SkDGCommand;

/// A command that does a skill roll. It follows the BRP 2023 rule book.
pub struct SkBRPCommand;

impl SkillHelper {
    /// Does a skill roll following the rule of Call of Cthulhu 7th Edition.
    async fn execute_7th(ctx: &Context, interaction: &CommandInteraction) -> Result<CommandStatus> {
        let chance = interaction.get_int_option("chance".to_string()).unwrap();

        let comment = interaction
            .get_string_option("comment".to_string())
            .unwrap_or("기능");

        let (result, roll) = match rand::thread_rng().gen_range(1..=100) {
            result if (result == 1 && result <= chance) => (
                ":star::crown::star: **대성공!!!**",
                format!("1 <= {}", chance),
            ),
            result if result <= chance / 5 => (
                ":crown: **극단적 성공!**",
                format!("{} <= {} / 5", result, chance),
            ),
            result if result <= chance / 2 => (
                ":o: **어려운 성공!**",
                format!("{} <= {} / 2", result, chance),
            ),
            result if result == 100 || (result > 95 && chance < 50) => {
                (":skull: **대실패!!!**", format!("{} >= {}", result, chance))
            }
            result if result <= chance => (":o: **보통 성공**", format!("{} <= {}", result, chance)),
            result => (":x: **실패**", format!("{} > {}", result, chance)),
        };

        interaction
            .send_embed(
                ctx,
                CreateEmbed::new()
                    .title(format!("{}의 {} 판정", interaction.get_nickname(), comment))
                    .field(result, roll, false),
            )
            .await?;

        Ok(CommandStatus::Ok)
    }

    async fn execute_dg(ctx: &Context, interaction: &CommandInteraction) -> Result<CommandStatus> {
        let chance = interaction.get_int_option("chance".to_string()).unwrap();

        let comment = interaction
            .get_string_option("comment".to_string())
            .unwrap_or("기능");

        let (result, roll) = match rand::thread_rng().gen_range(1..=100) {
            result if (result == 1 && result <= chance) => (
                ":star::crown::star: **대성공!!!**",
                format!("1 <= {}", chance),
            ),
            result if result <= 5 && (result <= chance) => {
                (":crown: **대성공!**", format!("{} <= {}", result, chance))
            }
            result if result / 10 == result % 10 && result <= chance => {
                (":crown: **대성공!**", format!("{} <= {}", result, chance))
            }
            result if result == 100 && (result > chance) => (
                ":fire::skull::fire: **펌블!!!**",
                format!("100 > {}", chance),
            ),
            result if result > 95 && (result > chance) => {
                (":skull: **대실패!!!**", format!("{} > {}", result, chance))
            }
            result if result <= chance => (":o: **보통 성공**", format!("{} <= {}", result, chance)),
            result => (":x: **실패**", format!("{} > {}", result, chance)),
        };

        interaction
            .send_embed(
                ctx,
                CreateEmbed::new()
                    .title(format!("{}의 {} 판정", interaction.get_nickname(), comment))
                    .field(result, roll, false),
            )
            .await?;

        Ok(CommandStatus::Ok)
    }

    async fn execute_brp(ctx: &Context, interaction: &CommandInteraction) -> Result<CommandStatus> {
        let chance = interaction.get_int_option("chance".to_string()).unwrap();

        let comment = interaction
            .get_string_option("comment".to_string())
            .unwrap_or("기능");

        let (result, roll) = match rand::thread_rng().gen_range(1..=100) {
            result if result <= (chance - 1) / 20 + 1 => (
                ":star::crown::star: **대성공!!!**",
                format!("{} <= {}", result, chance),
            ),
            result if result <= (chance - 1) / 5 + 1 => {
                (":crown: **특수 성공!**", format!("{} <= {}", result, chance))
            }
            result if result >= i32::min(96 + (chance - 1) / 20, 100) && (result > chance) => {
                (":skull: **대실패!!!**", format!("{} > {}", result, chance))
            }
            result if result <= chance => (":o: **보통 성공**", format!("{} <= {}", result, chance)),
            result => (":x: **실패**", format!("{} > {}", result, chance)),
        };

        interaction
            .send_embed(
                ctx,
                CreateEmbed::new()
                    .title(format!("{}의 {} 판정", interaction.get_nickname(), comment))
                    .field(result, roll, false),
            )
            .await?;

        Ok(CommandStatus::Ok)
    }
}

#[naming]
#[serenity::async_trait]
impl BotCommand for Sk7Command {
    fn create(&self) -> CreateCommand {
        CreateCommand::new(self.name())
            .description("크툴루의 부름 7판 룰에 따라 기능 판정을 합니다.")
            .add_option(
                CreateCommandOption::new(CommandOptionType::Integer, "chance", "기능 수치")
                    .required(true),
            )
            .add_option(
                CreateCommandOption::new(CommandOptionType::String, "comment", "판정 설명"),
            )
    }

    async fn execute(
        &self,
        ctx: &Context,
        interaction: &CommandInteraction,
    ) -> Result<CommandStatus> {
        SkillHelper::execute_7th(ctx, interaction).await
    }
}

#[naming]
#[serenity::async_trait]
impl BotCommand for SkDGCommand {
    fn create(&self) -> CreateCommand {
        CreateCommand::new(self.name())
            .description("델타 그린 룰에 따라 기능 판정을 합니다.")
            .add_option(
                CreateCommandOption::new(CommandOptionType::Integer, "chance", "기능 수치")
                    .required(true),
            )
            .add_option(
                CreateCommandOption::new(CommandOptionType::String, "comment", "판정 설명"),
            )
    }

    async fn execute(
        &self,
        ctx: &Context,
        interaction: &CommandInteraction,
    ) -> Result<CommandStatus> {
        SkillHelper::execute_dg(ctx, interaction).await
    }
}

#[naming]
#[serenity::async_trait]
impl BotCommand for SkBRPCommand {
    fn create(&self) -> CreateCommand {
        CreateCommand::new(self.name())
            .description("BRP 2023 룰에 따라 기능 판정을 합니다.")
            .add_option(
                CreateCommandOption::new(CommandOptionType::Integer, "chance", "기능 수치")
                    .required(true),
            )
            .add_option(
                CreateCommandOption::new(CommandOptionType::String, "comment", "판정 설명"),
            )
    }

    async fn execute(
        &self,
        ctx: &Context,
        interaction: &CommandInteraction,
    ) -> Result<CommandStatus> {
        SkillHelper::execute_brp(ctx, interaction).await
    }
}
