use log::info;
use serenity::all::{Interaction, Message, Ready};
use serenity::prelude::*;
use crate::commands::create_sheet::SheetStore;
use crate::webhook::send_webhook;
use crate::commands::BotCommandManager;

pub struct BotHandler;

#[serenity::async_trait]
impl EventHandler for BotHandler {
    async fn message(&self, ctx: Context, msg: Message) {
    // 봇의 메시지거나, 슬래시 명령어(/), 혹은 특수 문자로 시작하면 무시 (OOC 방지 등)
    if msg.author.bot || msg.content.starts_with('/') || msg.content.starts_with('!') || msg.content.starts_with('(') {
        return;
    }

    // 캐릭터 시트 보유 여부 및 이름 조회
    let has_character = {
        let data = ctx.data.read().await;
        if let Some(store) = data.get::<SheetStore>() {
            let sheets = store.read().await;
            if let Some(sheet) = sheets.get(&msg.author.id.to_string()) {
                if !sheet.name.is_empty() {
                    Some(sheet.name.clone())
                } else { None }
            } else { None }
        } else { None }
    };

    if let Some(char_name) = has_character {
        let avatar = msg.author.avatar_url();
        
        // 원본 메시지를 지우고, 캐릭터의 이름으로 웹훅 전송
        let _ = msg.delete(&ctx).await;
        let _ = send_webhook(&ctx, msg.channel_id, &char_name, avatar.as_deref(), &msg.content).await;
    }
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::Command(command) = interaction {
            let _ = BotCommandManager::run_command(&ctx, &command).await;
        }
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        info!("{} is connected!", ready.user.name);
        let _ = BotCommandManager::register_all(&ctx).await;
    }
}
