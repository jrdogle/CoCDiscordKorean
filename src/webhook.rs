use anyhow::Result;
use serenity::builder::{CreateWebhook, ExecuteWebhook};
use serenity::model::id::ChannelId;
use serenity::prelude::Context;

/// Webhook을 이용하여 메시지를 전송합니다.
pub async fn send_webhook(
    ctx: &Context,
    channel_id: ChannelId,
    username: &str,
    avatar_url: Option<&str>,
    content: &str,
) -> Result<()> {
    let webhooks = channel_id.webhooks(ctx).await?;
    let target_webhook = webhooks
        .into_iter()
        .find(|w| w.name.as_deref() == Some("cthulhu_webhook"));

    let webhook = match target_webhook {
        Some(w) => w,
        None => channel_id.create_webhook(ctx, CreateWebhook::new("cthulhu_webhook")).await?,
    };

    let mut builder = ExecuteWebhook::new().content(content).username(username);
    if let Some(avatar) = avatar_url {
        builder = builder.avatar_url(avatar);
    }

    webhook.execute(ctx, false, builder).await?;
    Ok(())
}