use anyhow::Result;
use serenity::builder::{CreateCommand, CreateCommandOption, CreateEmbed};
use serenity::model::application::{CommandInteraction, CommandOptionType};
use serenity::prelude::Context;

use crate::commands::{BotCommand, CommandStatus, InteractionUtil, SendEmbed};

/// A command that creates a character sheet.
pub struct CSCommand;

#[naming]
#[serenity::async_trait]
impl BotCommand for CSCommand {
    fn create(&self) -> CreateCommand {
        CreateCommand::new(self.name())
            .description("탐사자 시트를 생성합니다.")
            .add_option(CreateCommandOption::new(CommandOptionType::Integer, "str", "근력 (STR)").required(true))
            .add_option(CreateCommandOption::new(CommandOptionType::Integer, "con", "건강 (CON)").required(true))
            .add_option(CreateCommandOption::new(CommandOptionType::Integer, "siz", "크기 (SIZ)").required(true))
            .add_option(CreateCommandOption::new(CommandOptionType::Integer, "dex", "민첩 (DEX)").required(true))
            .add_option(CreateCommandOption::new(CommandOptionType::Integer, "app", "외모 (APP)").required(true))
            .add_option(CreateCommandOption::new(CommandOptionType::Integer, "int", "지능 (INT)").required(true))
            .add_option(CreateCommandOption::new(CommandOptionType::Integer, "pow", "정신 (POW)").required(true))
            .add_option(CreateCommandOption::new(CommandOptionType::Integer, "edu", "교육 (EDU)").required(true))
    }

    async fn execute(
        &self,
        ctx: &Context,
        interaction: &CommandInteraction,
    ) -> Result<CommandStatus> {
        let author = interaction.get_nickname();

        let str_val = interaction.get_int_option("근력".into()).unwrap();
        let con_val = interaction.get_int_option("건강".into()).unwrap();
        let siz_val = interaction.get_int_option("크기".into()).unwrap();
        let dex_val = interaction.get_int_option("민첩".into()).unwrap();
        let app_val = interaction.get_int_option("외모".into()).unwrap();
        let int_val = interaction.get_int_option("지능".into()).unwrap();
        let pow_val = interaction.get_int_option("정신".into()).unwrap();
        let edu_val = interaction.get_int_option("교육".into()).unwrap();

        let embed = CreateEmbed::new()
            .title(format!("{}의 탐사자", author))
            .field(":muscle: 근력", str_val.to_string(), true)
            .field(":shield: 건강", con_val.to_string(), true)
            .field(":straight_ruler: 크기", siz_val.to_string(), true)
            .field(":runner: 민첩", dex_val.to_string(), true)
            .field(":sparkles: 외모", app_val.to_string(), true)
            .field(":bulb: 지능", int_val.to_string(), true)
            .field(":crystal_ball: 정신", pow_val.to_string(), true)
            .field(":mortar_board: 교육", edu_val.to_string(), true);

        interaction.send_embed(ctx, embed).await?;

        Ok(CommandStatus::Ok)
    }
}
