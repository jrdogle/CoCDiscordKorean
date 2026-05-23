use anyhow::Result;
use rand::Rng;
use serenity::builder::{CreateCommand, CreateCommandOption, CreateEmbed};
use serenity::model::application::{CommandInteraction, CommandOptionType};
use serenity::prelude::Context;

use crate::commands::create_sheet::{save_sheets, SheetStore};
use crate::commands::{BotCommand, CommandStatus, InteractionUtil, SendEmbed};
use crate::commands::roll::RollCommand;

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

#[serenity::async_trait]
impl BotCommand for SkillCommand {
    fn name(&self) -> &str {
        "판정"
    }

    fn create(&self) -> CreateCommand {
        CreateCommand::new(self.name())
            .description("크툴루의 부름 7판 룰에 따라 기능 판정을 합니다.")
            .add_option(
                CreateCommandOption::new(CommandOptionType::Integer, "수치", "판정하려는 기능 수치")
                    .required(false),
            )
            .add_option(
                CreateCommandOption::new(CommandOptionType::String, "특성치", "시트에 저장된 특성치로 판정")
                    .add_string_choice("근력", "str")
                    .add_string_choice("건강", "con")
                    .add_string_choice("크기", "siz")
                    .add_string_choice("민첩", "dex")
                    .add_string_choice("외모", "app")
                    .add_string_choice("지능", "int")
                    .add_string_choice("정신", "pow")
                    .add_string_choice("교육", "edu")
                    .add_string_choice("체력", "hp")
                    .add_string_choice("마력", "mp")
                    .add_string_choice("운", "luck")
                    .add_string_choice("이성", "san")
                    .required(false),
            )
            .add_option(
                CreateCommandOption::new(CommandOptionType::String, "기능이름", "판정하려는 기능 이름"),
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

        if let Some(val) = interaction.get_int_option("수치".to_string()) {
            chance_val = Some(val);
        }

        let stat_opt = interaction.get_string_option("특성치".to_string());

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
                            "str" => { chance_val = Some(sheet.str_val); stat_name_display = "근력".to_string(); }
                            "con" => { chance_val = Some(sheet.con_val); stat_name_display = "건강".to_string(); }
                            "siz" => { chance_val = Some(sheet.siz_val); stat_name_display = "크기".to_string(); }
                            "dex" => { chance_val = Some(sheet.dex_val); stat_name_display = "민첩".to_string(); }
                            "app" => { chance_val = Some(sheet.app_val); stat_name_display = "외모".to_string(); }
                            "int" => { chance_val = Some(sheet.int_val); stat_name_display = "지능".to_string(); }
                            "pow" => { chance_val = Some(sheet.pow_val); stat_name_display = "정신".to_string(); }
                            "edu" => { chance_val = Some(sheet.edu_val); stat_name_display = "교육".to_string(); }
                            "hp" => { chance_val = Some(sheet.hp); stat_name_display = "체력".to_string(); }
                            "mp" => { chance_val = Some(sheet.mp); stat_name_display = "마력".to_string(); }
                            "luck" => { chance_val = Some(sheet.luck); stat_name_display = "운".to_string(); }
                            "san" => { chance_val = Some(sheet.san); stat_name_display = "이성".to_string(); }
                            _ => {}
                        }
                    }
                } else if stat_opt.is_some() {
                    return Ok(CommandStatus::Err("저장된 캐릭터 시트가 없습니다. `/시트생성` 명령어로 먼저 시트를 생성해 주세요.".to_string()));
                }
            }
        }

        let chance = match chance_val {
            Some(v) => v,
            None => return Ok(CommandStatus::Err("기능 수치(`수치`)를 직접 입력하거나 판정할 특성치(`특성치`)를 선택해야 합니다.".to_string())),
        };

        let comment = interaction
            .get_string_option("기능이름".to_string())
            .unwrap_or(if stat_name_display.is_empty() { "기능" } else { &stat_name_display });

        SkillHelper::execute_7th(ctx, interaction, chance, comment, &character_name).await
    }
}

/// A command that consumes luck to change a failed roll into a success.
pub struct UseLuckCommand;

#[serenity::async_trait]
impl BotCommand for UseLuckCommand {
    fn name(&self) -> &str {
        "운소모"
    }

    fn create(&self) -> CreateCommand {
        CreateCommand::new(self.name())
            .description("운을 소모하여 실패한 판정을 원하는 성공 수준으로 수정합니다.")
            .add_option(CreateCommandOption::new(CommandOptionType::Integer, "주사위값", "나온 주사위 값 (예: 52)").required(true))
            .add_option(
                CreateCommandOption::new(CommandOptionType::String, "목표성공", "달성하고자 하는 성공 수준")
                    .add_string_choice("보통 성공", "regular")
                    .add_string_choice("어려운 성공", "hard")
                    .add_string_choice("극단적 성공", "extreme")
                    .add_string_choice("대성공", "critical")
                    .required(true)
            )
            .add_option(CreateCommandOption::new(CommandOptionType::Integer, "기능수치", "판정했던 기능의 원래 수치 (직접 입력 시)").required(false))
            .add_option(
                CreateCommandOption::new(CommandOptionType::String, "특성치", "시트에 저장된 특성치로 판정했던 경우")
                    .add_string_choice("근력", "str")
                    .add_string_choice("건강", "con")
                    .add_string_choice("크기", "siz")
                    .add_string_choice("민첩", "dex")
                    .add_string_choice("외모", "app")
                    .add_string_choice("지능", "int")
                    .add_string_choice("정신", "pow")
                    .add_string_choice("교육", "edu")
                    .add_string_choice("체력", "hp")
                    .add_string_choice("마력", "mp")
                    .add_string_choice("운", "luck")
                    .add_string_choice("이성", "san")
                    .required(false),
            )
    }

    async fn execute(&self, ctx: &Context, interaction: &CommandInteraction) -> Result<CommandStatus> {
        let roll = interaction.get_int_option("주사위값".into()).unwrap();
        let target_level = interaction.get_string_option("목표성공".into()).unwrap();
        let user_id = interaction.user.id;

        let mut chance_val = interaction.get_int_option("기능수치".into());
        let stat_opt = interaction.get_string_option("특성치".into());
        let mut character_name = interaction.get_nickname();
        
        let mut current_luck = 0;
        let mut success = false;

        let store = {
            let data = ctx.data.read().await;
            data.get::<SheetStore>().cloned()
        };

        if stat_opt.is_some() || chance_val.is_none() {
            if let Some(store) = &store {
                let sheets = store.read().await;
                if let Some(sheet) = sheets.get(&user_id.to_string()) {
                    if !sheet.name.is_empty() { character_name = sheet.name.clone(); }
                    if let Some(stat) = stat_opt {
                        match stat {
                            "str" => chance_val = Some(sheet.str_val),
                            "con" => chance_val = Some(sheet.con_val),
                            "siz" => chance_val = Some(sheet.siz_val),
                            "dex" => chance_val = Some(sheet.dex_val),
                            "app" => chance_val = Some(sheet.app_val),
                            "int" => chance_val = Some(sheet.int_val),
                            "pow" => chance_val = Some(sheet.pow_val),
                            "edu" => chance_val = Some(sheet.edu_val),
                            "hp" => chance_val = Some(sheet.hp),
                            "mp" => chance_val = Some(sheet.mp),
                            "luck" => chance_val = Some(sheet.luck),
                            "san" => chance_val = Some(sheet.san),
                            _ => {}
                        }
                    }
                }
            }
        }

        let chance = match chance_val {
            Some(v) => v,
            None => return Ok(CommandStatus::Err("기능 수치(`기능수치`)를 직접 입력하거나 판정할 특성치(`특성치`)를 선택해야 합니다.".to_string())),
        };

        let (target, level_str, icon) = match target_level {
            "critical" => (1, "대성공", ":star::crown::star:"),
            "extreme" => (chance / 5, "극단적 성공", ":crown:"),
            "hard" => (chance / 2, "어려운 성공", ":o:"),
            _ => (chance, "보통 성공", ":o:"),
        };

        let cost = roll - target;
        if cost <= 0 {
            return Ok(CommandStatus::Err(format!("수정할 필요 없이 이미 **{}** 이상을 달성한 수치입니다! (현재 주사위: {}, 목표: {})", level_str, roll, target)));
        }

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
                return Ok(CommandStatus::Err("저장된 캐릭터 시트가 없습니다. `/시트생성` 명령어로 먼저 시트를 생성해 주세요.".to_string()));
            }
        }

        if success {
            let embed = CreateEmbed::new()
                .title(format!("{}의 운 소모!", character_name))
                .description(format!("운을 **{}** 소모하여 판정을 수정했습니다!\n(남은 운: **{}**)", cost, current_luck))
                .field("판정 결과", format!("{} ➔ **{}** ({} {})", roll, target, icon, level_str), false);

            interaction.send_embed(ctx, embed).await?;
            Ok(CommandStatus::Ok)
        } else {
            Ok(CommandStatus::Err("시스템 오류로 운을 소모하지 못했습니다.".to_string()))
        }
    }
}

/// A command that does a sanity roll and deducts SAN.
pub struct SanRollCommand;

#[serenity::async_trait]
impl BotCommand for SanRollCommand {
    fn name(&self) -> &str {
        "이성판정"
    }

    fn create(&self) -> CreateCommand {
        CreateCommand::new(self.name())
            .description("이성(SAN) 판정을 하고 결과에 따라 이성을 감소시킵니다.")
            .add_option(CreateCommandOption::new(CommandOptionType::String, "성공시감소량", "성공 시 감소량 (예: 0, 1, 1d3)").required(true))
            .add_option(CreateCommandOption::new(CommandOptionType::String, "실패시감소량", "실패 시 감소량 (예: 1, 1d6, 1d10)").required(true))
    }

    async fn execute(&self, ctx: &Context, interaction: &CommandInteraction) -> Result<CommandStatus> {
        let success_loss = interaction.get_string_option("성공시감소량".into()).unwrap();
        let failure_loss = interaction.get_string_option("실패시감소량".into()).unwrap();
        let user_id = interaction.user.id;

        let mut character_name = interaction.get_nickname();
        let mut current_san = 0;
        let mut has_sheet = false;

        let store = {
            let data = ctx.data.read().await;
            data.get::<SheetStore>().cloned()
        };

        if let Some(store) = &store {
            let sheets = store.read().await;
            if let Some(sheet) = sheets.get(&user_id.to_string()) {
                if !sheet.name.is_empty() {
                    character_name = sheet.name.clone();
                }
                current_san = sheet.san;
                has_sheet = true;
            }
        }

        if !has_sheet {
            return Ok(CommandStatus::Err("저장된 캐릭터 시트가 없습니다. `/시트생성` 명령어로 먼저 시트를 생성해 주세요.".to_string()));
        }

        let roll = {
            let mut rng = rand::thread_rng();
            rng.gen_range(1..=100)
        };

        let is_success = roll <= current_san;
        let expr = if is_success { success_loss } else { failure_loss };

        let (loss_amount, dice_desc) = match RollCommand::evaluate_dice_expr(expr) {
            Ok(res) => res,
            Err(msg) => return Ok(CommandStatus::Err(format!("주사위 식 오류: {}", msg))),
        };

        let loss_amount = i32::max(0, loss_amount);
        let mut new_san = current_san;
        let mut madness_warning = false;

        if let Some(store) = store {
            let mut sheets = store.write().await;
            if let Some(sheet) = sheets.get_mut(&user_id.to_string()) {
                sheet.san -= loss_amount;
                new_san = sheet.san;
                if sheet.san < (sheet.pow_val * 80 / 100) {
                    madness_warning = true;
                }
                save_sheets(&sheets).await;
            }
        }

        let result_text = if is_success {
            format!(":o: **성공** ({} <= {})", roll, current_san)
        } else {
            format!(":x: **실패** ({} > {})", roll, current_san)
        };

        let mut desc = format!("**감소치 굴림:** {} ➔ **{}** 감소\n**남은 이성:** {} ➔ **{}**", dice_desc, loss_amount, current_san, new_san);

        if madness_warning {
            desc.push_str("\n\n:warning: **이성이 정신 스탯의 80% 미만으로 떨어져 광기에 걸렸습니다!**");
        }

        let embed = CreateEmbed::new()
            .title(format!("{}의 이성 판정", character_name))
            .field("판정 결과", result_text, false)
            .field("이성 감소", desc, false);

        interaction.send_embed(ctx, embed).await?;
        Ok(CommandStatus::Ok)
    }
}
