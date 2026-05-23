use anyhow::Result;
use rand::Rng;
use serenity::builder::{CreateCommand, CreateCommandOption, CreateEmbed};
use serenity::model::application::{CommandInteraction, CommandOptionType};
use serenity::prelude::Context;

use crate::commands::create_sheet::{save_sheets, SheetStore};
use crate::commands::{BotCommand, CommandStatus, InteractionUtil, SendEmbed};

/// A helper for skill rolls.
struct SkillHelper;

/// A command that does a skill roll. It follows Call of Cthulhu 7th Edition.
pub struct SkillCommand;

impl SkillHelper {
    /// Does a skill roll following the rule of Call of Cthulhu 7th Edition.
    async fn execute_7th(ctx: &Context, interaction: &CommandInteraction, chance: i32, comment: &str, character_name: &str) -> Result<CommandStatus> {
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
                    .title(format!("{}의 {} 판정", character_name, comment))
                    .field(result, roll, false),
            )
            .await?;

        Ok(CommandStatus::Ok)
    }
}

#[naming]
#[serenity::async_trait]
impl BotCommand for SkillCommand {
    fn create(&self) -> CreateCommand {
        CreateCommand::new(self.name())
            .description("크툴루의 부름 7판 룰에 따라 기능 판정을 합니다.")
            .add_option(
                CreateCommandOption::new(CommandOptionType::Integer, "chance", "기능 수치")
                    .required(false),
            )
            .add_option(
                CreateCommandOption::new(CommandOptionType::String, "stat", "시트에 저장된 특성치로 판정")
                    .add_string_choice("근력 (STR)", "str")
                    .add_string_choice("건강 (CON)", "con")
                    .add_string_choice("크기 (SIZ)", "siz")
                    .add_string_choice("민첩 (DEX)", "dex")
                    .add_string_choice("외모 (APP)", "app")
                    .add_string_choice("지능 (INT)", "int")
                    .add_string_choice("정신 (POW)", "pow")
                    .add_string_choice("교육 (EDU)", "edu")
                    .add_string_choice("체력 (HP)", "hp")
                    .add_string_choice("마력 (MP)", "mp")
                    .add_string_choice("운 (Luck)", "luck")
                    .required(false),
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
        let mut chance_val = None;
        let mut stat_name_display = String::new();
        let mut character_name = interaction.get_nickname();

        if let Some(val) = interaction.get_int_option("chance".to_string()) {
            chance_val = Some(val);
        }

        let stat_opt = interaction.get_string_option("stat".to_string());

        if stat_opt.is_some() || chance_val.is_none() {
            let user_id = interaction.user.id;
            let store = {
                let data = ctx.data.read().await;
                data.get::<SheetStore>().cloned()
            };
            if let Some(store) = store {
                let sheets = store.read().await;
                if let Some(sheet) = sheets.get(&user_id.to_string()) {
                    if !sheet.name.is_empty() { character_name = sheet.name.clone(); }
                    if let Some(stat) = stat_opt {
                        match stat {
                            "str" => { chance_val = Some(sheet.str_val); stat_name_display = "근력 (STR)".to_string(); }
                            "con" => { chance_val = Some(sheet.con_val); stat_name_display = "건강 (CON)".to_string(); }
                            "siz" => { chance_val = Some(sheet.siz_val); stat_name_display = "크기 (SIZ)".to_string(); }
                            "dex" => { chance_val = Some(sheet.dex_val); stat_name_display = "민첩 (DEX)".to_string(); }
                            "app" => { chance_val = Some(sheet.app_val); stat_name_display = "외모 (APP)".to_string(); }
                            "int" => { chance_val = Some(sheet.int_val); stat_name_display = "지능 (INT)".to_string(); }
                            "pow" => { chance_val = Some(sheet.pow_val); stat_name_display = "정신 (POW)".to_string(); }
                            "edu" => { chance_val = Some(sheet.edu_val); stat_name_display = "교육 (EDU)".to_string(); }
                            "hp" => { chance_val = Some(sheet.hp); stat_name_display = "체력 (HP)".to_string(); }
                            "mp" => { chance_val = Some(sheet.mp); stat_name_display = "마력 (MP)".to_string(); }
                            "luck" => { chance_val = Some(sheet.luck); stat_name_display = "운 (Luck)".to_string(); }
                            _ => {}
                        }
                    }
                } else if stat_opt.is_some() {
                    return Ok(CommandStatus::Err("저장된 캐릭터 시트가 없습니다. `/cs` 명령어로 먼저 시트를 생성해 주세요.".to_string()));
                }
            }
        }

        let chance = match chance_val {
            Some(v) => v,
            None => return Ok(CommandStatus::Err("기능 수치(`chance`)를 직접 입력하거나 판정할 특성치(`stat`)를 선택해야 합니다.".to_string())),
        };

        let comment = interaction
            .get_string_option("comment".to_string())
            .unwrap_or(if stat_name_display.is_empty() { "기능" } else { &stat_name_display });

        SkillHelper::execute_7th(ctx, interaction, chance, comment, &character_name).await
    }
}

/// A command that consumes luck to change a failed roll into a success.
pub struct UseLuckCommand;

#[naming]
#[serenity::async_trait]
impl BotCommand for UseLuckCommand {
    fn create(&self) -> CreateCommand {
        CreateCommand::new(self.name())
            .description("운을 소모하여 실패한 판정을 보통 성공으로 수정합니다.")
            .add_option(CreateCommandOption::new(CommandOptionType::Integer, "roll", "나온 주사위 값 (예: 52)").required(true))
            .add_option(CreateCommandOption::new(CommandOptionType::Integer, "target", "목표했던 성공 수치 (예: 50)").required(true))
    }

    async fn execute(&self, ctx: &Context, interaction: &CommandInteraction) -> Result<CommandStatus> {
        let roll = interaction.get_int_option("roll".into()).unwrap();
        let target = interaction.get_int_option("target".into()).unwrap();
        let user_id = interaction.user.id;

        let cost = roll - target;
        if cost <= 0 {
            return Ok(CommandStatus::Err("수정할 필요 없이 이미 성공한 수치입니다!".to_string()));
        }

        let mut current_luck = 0;
        let mut success = false;
        let mut character_name = interaction.get_nickname();

        let store = {
            let data = ctx.data.read().await;
            data.get::<SheetStore>().cloned()
        };

        if let Some(store) = store {
            let mut sheets = store.write().await;
            if let Some(sheet) = sheets.get_mut(&user_id.to_string()) {
                if !sheet.name.is_empty() {
                    character_name = sheet.name.clone();
                }
                if sheet.luck >= cost {
                    sheet.luck -= cost;
                    current_luck = sheet.luck;
                    success = true;
                    save_sheets(&sheets).await;
                } else {
                    return Ok(CommandStatus::Err(format!("운이 부족합니다! (현재 운: {}, 필요 운: {})", sheet.luck, cost)));
                }
            } else {
                return Ok(CommandStatus::Err("저장된 캐릭터 시트가 없습니다. `/cs` 명령어로 먼저 시트를 생성해 주세요.".to_string()));
            }
        }

        if success {
            let embed = CreateEmbed::new()
                .title(format!("{}의 운 소모!", character_name))
                .description(format!("운을 **{}** 소모하여 판정을 수정했습니다!\n(남은 운: **{}**)", cost, current_luck))
                .field("판정 결과", format!("{} ➔ **{}** (보통 성공)", roll, target), false);

            interaction.send_embed(ctx, embed).await?;
            Ok(CommandStatus::Ok)
        } else {
            Ok(CommandStatus::Err("시스템 오류로 운을 소모하지 못했습니다.".to_string()))
        }
    }
}
