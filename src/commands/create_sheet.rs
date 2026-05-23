use std::collections::HashMap;
use std::sync::Arc;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use serenity::builder::{CreateCommand, CreateCommandOption, CreateEmbed, CreateInteractionResponse, CreateInteractionResponseMessage};
use serenity::model::application::{CommandInteraction, CommandOptionType};
use serenity::prelude::Context;
use serenity::prelude::TypeMapKey;
use tokio::sync::RwLock;

use crate::commands::{BotCommand, CommandStatus, InteractionUtil, SendEmbed};

#[derive(Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct CharacterSheet {
    pub name: String,
    pub str_val: i32,
    pub con_val: i32,
    pub siz_val: i32,
    pub dex_val: i32,
    pub app_val: i32,
    pub int_val: i32,
    pub pow_val: i32,
    pub edu_val: i32,
    pub hp: i32,
    pub hp_max: i32,
    pub mp: i32,
    pub mp_max: i32,
    pub luck: i32,
    pub san: i32,
}

pub struct SheetStore;
impl TypeMapKey for SheetStore {
    type Value = Arc<RwLock<HashMap<String, CharacterSheet>>>;
}

pub fn load_sheets() -> HashMap<String, CharacterSheet> {
    let file_path = std::path::Path::new("sheets.json");
    if let Ok(json) = std::fs::read_to_string(file_path) {
        return serde_json::from_str(&json).unwrap_or_default();
    }
    HashMap::new()
}

pub async fn save_sheets(sheets: &HashMap<String, CharacterSheet>) {
    if let Ok(json) = serde_json::to_string_pretty(sheets) {
        let file_path = std::path::Path::new("sheets.json");
        let _ = tokio::fs::write(file_path, json).await;
    }
}

/// A command that creates a character sheet.
pub struct CSCommand;

#[serenity::async_trait]
impl BotCommand for CSCommand {
    fn name(&self) -> &str {
        "시트생성"
    }

    fn create(&self) -> CreateCommand {
        CreateCommand::new(self.name())
            .description("탐사자 시트를 생성합니다.")
            .add_option(CreateCommandOption::new(CommandOptionType::String, "이름", "탐사자 이름").required(true))
            .add_option(CreateCommandOption::new(CommandOptionType::Integer, "근력", "근력").required(true))
            .add_option(CreateCommandOption::new(CommandOptionType::Integer, "건강", "건강").required(true))
            .add_option(CreateCommandOption::new(CommandOptionType::Integer, "크기", "크기").required(true))
            .add_option(CreateCommandOption::new(CommandOptionType::Integer, "민첩", "민첩").required(true))
            .add_option(CreateCommandOption::new(CommandOptionType::Integer, "외모", "외모").required(true))
            .add_option(CreateCommandOption::new(CommandOptionType::Integer, "지능", "지능").required(true))
            .add_option(CreateCommandOption::new(CommandOptionType::Integer, "정신", "정신").required(true))
            .add_option(CreateCommandOption::new(CommandOptionType::Integer, "교육", "교육").required(true))
            .add_option(CreateCommandOption::new(CommandOptionType::Integer, "운", "운").required(true))
    }

    async fn execute(
        &self,
        ctx: &Context,
        interaction: &CommandInteraction,
    ) -> Result<CommandStatus> {
        let user_id = interaction.user.id;

        let name = interaction.get_string_option("이름".into()).unwrap().to_string();
        let str_val = interaction.get_int_option("근력".into()).unwrap();
        let con_val = interaction.get_int_option("건강".into()).unwrap();
        let siz_val = interaction.get_int_option("크기".into()).unwrap();
        let dex_val = interaction.get_int_option("민첩".into()).unwrap();
        let app_val = interaction.get_int_option("외모".into()).unwrap();
        let int_val = interaction.get_int_option("지능".into()).unwrap();
        let pow_val = interaction.get_int_option("정신".into()).unwrap();
        let edu_val = interaction.get_int_option("교육".into()).unwrap();
        let luck = interaction.get_int_option("운".into()).unwrap();

        let hp_max = (con_val + siz_val) / 10;
        let mp_max = pow_val / 5;
        let san = pow_val;

        let sheet = CharacterSheet { 
            name: name.clone(), str_val, con_val, siz_val, dex_val, app_val, int_val, pow_val, edu_val, hp: hp_max, hp_max, mp: mp_max, mp_max, luck, san 
        };

        let store = {
            let data = ctx.data.read().await;
            data.get::<SheetStore>().cloned()
        };
        if let Some(store) = store {
            let mut sheets = store.write().await;
            sheets.insert(user_id.to_string(), sheet);
            save_sheets(&sheets).await;
        }

        let embed = CreateEmbed::new()
            .title(format!("탐사자: {}", name))
            .field(":muscle: 근력", str_val.to_string(), true)
            .field(":shield: 건강", con_val.to_string(), true)
            .field(":straight_ruler: 크기", siz_val.to_string(), true)
            .field(":runner: 민첩", dex_val.to_string(), true)
            .field(":sparkles: 외모", app_val.to_string(), true)
            .field(":bulb: 지능", int_val.to_string(), true)
            .field(":crystal_ball: 정신", pow_val.to_string(), true)
            .field(":mortar_board: 교육", edu_val.to_string(), true)
            .field(":heart: 체력", format!("{}/{}", hp_max, hp_max), true)
            .field(":star: 마력", format!("{}/{}", mp_max, mp_max), true)
            .field(":four_leaf_clover: 운", luck.to_string(), true)
            .field(":brain: 이성", san.to_string(), true);

        interaction.send_embed(ctx, embed).await?;

        Ok(CommandStatus::Ok)
    }
}

/// A command that shows the saved character sheet.
pub struct ShowSheetCommand;

#[serenity::async_trait]
impl BotCommand for ShowSheetCommand {
    fn name(&self) -> &str {
        "시트보기"
    }

    fn create(&self) -> CreateCommand {
        CreateCommand::new(self.name()).description("내 탐사자 시트 현황을 확인합니다.")
    }

    async fn execute(&self, ctx: &Context, interaction: &CommandInteraction) -> Result<CommandStatus> {
        let user_id = interaction.user.id;
        let author = interaction.get_nickname();

        let sheet = {
            let store = {
                let data = ctx.data.read().await;
                data.get::<SheetStore>().cloned()
            };
            if let Some(store) = store {
                store.read().await.get(&user_id.to_string()).cloned()
            } else {
                None
            }
        };

        if let Some(s) = sheet {
            let display_name = if s.name.is_empty() { author } else { s.name.clone() };
            let embed = CreateEmbed::new()
                .title(format!("탐사자: {}", display_name))
                .field(":muscle: 근력", s.str_val.to_string(), true)
                .field(":shield: 건강", s.con_val.to_string(), true)
                .field(":straight_ruler: 크기", s.siz_val.to_string(), true)
                .field(":runner: 민첩", s.dex_val.to_string(), true)
                .field(":sparkles: 외모", s.app_val.to_string(), true)
                .field(":bulb: 지능", s.int_val.to_string(), true)
                .field(":crystal_ball: 정신", s.pow_val.to_string(), true)
                .field(":mortar_board: 교육", s.edu_val.to_string(), true)
                .field(":heart: 체력", format!("{}/{}", s.hp, s.hp_max), true)
                .field(":star: 마력", format!("{}/{}", s.mp, s.mp_max), true)
                .field(":four_leaf_clover: 운", s.luck.to_string(), true)
                .field(":brain: 이성", s.san.to_string(), true);

            interaction.create_response(
                ctx,
                CreateInteractionResponse::Message(
                    CreateInteractionResponseMessage::new().add_embed(embed).ephemeral(true)
                )
            ).await?;
            Ok(CommandStatus::Ok)
        } else {
            interaction.create_response(
                ctx,
                CreateInteractionResponse::Message(
                    CreateInteractionResponseMessage::new().content("저장된 캐릭터 시트가 없습니다. `/시트생성` 명령어로 먼저 시트를 생성해 주세요.").ephemeral(true)
                )
            ).await?;
            Ok(CommandStatus::Ok)
        }
    }
}

/// A command that edits the name in the character sheet.
pub struct EditNameCommand;

#[serenity::async_trait]
impl BotCommand for EditNameCommand {
    fn name(&self) -> &str {
        "이름설정"
    }

    fn create(&self) -> CreateCommand {
        CreateCommand::new(self.name())
            .description("채팅에 표시될 내 캐릭터 이름을 설정하거나 변경합니다.")
            .add_option(CreateCommandOption::new(CommandOptionType::String, "이름", "새로운 캐릭터 이름").required(true))
    }

    async fn execute(&self, ctx: &Context, interaction: &CommandInteraction) -> Result<CommandStatus> {
        let new_name = interaction.get_string_option("이름".into()).unwrap().to_string();
        let user_id = interaction.user.id;

        let mut old_name = String::new();

        let store = {
            let data = ctx.data.read().await;
            data.get::<SheetStore>().cloned()
        };
        if let Some(store) = store {
            let mut sheets = store.write().await;
            let sheet = sheets.entry(user_id.to_string()).or_insert_with(CharacterSheet::default);
            old_name = sheet.name.clone();
            sheet.name = new_name.clone();
            save_sheets(&sheets).await;
        }

        let old_name_display = if old_name.is_empty() { "이름 없음".to_string() } else { old_name };
        let embed = CreateEmbed::new()
            .title("캐릭터 이름 설정 완료")
            .description(format!("캐릭터 이름이 **{}**에서 **{}**(으)로 변경되었습니다.\n이제 일반 채팅을 입력하면 이 캐릭터 이름으로 자동 출력됩니다.", old_name_display, new_name));
        interaction.send_embed(ctx, embed).await?;
        Ok(CommandStatus::Ok)
    }
}

/// A command that edits a specific status in the character sheet.
pub struct EditStatCommand;

#[serenity::async_trait]
impl BotCommand for EditStatCommand {
    fn name(&self) -> &str {
        "특성치수정"
    }

    fn create(&self) -> CreateCommand {
        CreateCommand::new(self.name())
            .description("내 탐사자 시트의 특정 특성치를 수정합니다.")
            .add_option(
                CreateCommandOption::new(CommandOptionType::String, "특성치", "수정할 특성치")
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
                    .required(true),
            )
            .add_option(CreateCommandOption::new(CommandOptionType::Integer, "수치", "새로운 수치").required(true))
    }

    async fn execute(&self, ctx: &Context, interaction: &CommandInteraction) -> Result<CommandStatus> {
        let stat = interaction.get_string_option("특성치".into()).unwrap();
        let value = interaction.get_int_option("수치".into()).unwrap();
        let user_id = interaction.user.id;

        let mut success = false;
        let mut stat_name = "";
        let mut madness_warning = false;
        let mut old_value = 0;
        let mut character_name = String::new();

        let store = {
            let data = ctx.data.read().await;
            data.get::<SheetStore>().cloned()
        };
        if let Some(store) = store {
            let mut sheets = store.write().await;
            if let Some(sheet) = sheets.get_mut(&user_id.to_string()) {
                success = true;
                character_name = sheet.name.clone();
                match stat {
                        "str" => { old_value = sheet.str_val; sheet.str_val = value; stat_name = "근력"; }
                        "con" => { old_value = sheet.con_val; sheet.con_val = value; stat_name = "건강"; }
                        "siz" => { old_value = sheet.siz_val; sheet.siz_val = value; stat_name = "크기"; }
                        "dex" => { old_value = sheet.dex_val; sheet.dex_val = value; stat_name = "민첩"; }
                        "app" => { old_value = sheet.app_val; sheet.app_val = value; stat_name = "외모"; }
                        "int" => { old_value = sheet.int_val; sheet.int_val = value; stat_name = "지능"; }
                        "pow" => { old_value = sheet.pow_val; sheet.pow_val = value; stat_name = "정신"; }
                        "edu" => { old_value = sheet.edu_val; sheet.edu_val = value; stat_name = "교육"; }
                        "hp" => { old_value = sheet.hp; sheet.hp = value; stat_name = "체력"; }
                        "mp" => { old_value = sheet.mp; sheet.mp = value; stat_name = "마력"; }
                        "luck" => { old_value = sheet.luck; sheet.luck = value; stat_name = "운"; }
                        "san" => { 
                            old_value = sheet.san;
                            sheet.san = value; 
                            stat_name = "이성"; 
                            if sheet.san < (sheet.pow_val * 80 / 100) {
                                madness_warning = true;
                            }
                        }
                        _ => { success = false; }
                    }
                save_sheets(&sheets).await;
            }
        }

        if success {
            let display_name = if character_name.is_empty() { interaction.get_nickname() } else { character_name };
            let mut desc = format!("**{}** 수치 변경: **{}** -> **{}**", stat_name, old_value, value);
            if madness_warning {
                desc.push_str("\n\n:warning: **이성이 정신 스탯의 80% 미만으로 떨어져 광기에 걸렸습니다!**");
            }
            let embed = CreateEmbed::new()
                .title(format!("{}의 특성치 수정 완료", display_name))
                .description(desc);
            interaction.send_embed(ctx, embed).await?;
            Ok(CommandStatus::Ok)
        } else {
            Ok(CommandStatus::Err("저장된 캐릭터 시트가 없습니다. `/시트생성` 명령어로 먼저 시트를 생성해 주세요.".to_string()))
        }
    }
}
