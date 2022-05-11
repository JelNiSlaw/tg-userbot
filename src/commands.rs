use grammers_client::client::chats::InvocationError;
use grammers_client::types::{Chat, Message};
use grammers_client::{Client, InputMessage};
use rand::prelude::SliceRandom;
use rand::Rng;

use crate::ZENON;

const RESPONSES: [&str; 9] = [
    "zamknij ryj",
    "bądź cicho",
    "cicho bądź",
    "przestań spamić",
    "super materiał (nie)",
    "ratio",
    "kto pytał",
    "nie pytałem",
    "jaki masz program że na każdy kanał wklejasz te treści ?",
];

pub struct Context {
    pub client: Client,
    pub message: Message,
    pub chat: Chat,
}

pub async fn strategia(ctx: &Context) -> Result<(), InvocationError> {
    let mut rng = rand::thread_rng();
    let mut messages = Vec::new();
    while rng.gen::<bool>() {
        messages.push(
            ctx.client
                .send_message(
                    &ctx.chat,
                    *["strategia", "strateg", "strategicznie"]
                        .choose(&mut rng)
                        .unwrap(),
                )
                .await?
                .id(),
        );
    }
    ctx.client.delete_messages(&ctx.chat, &messages).await?;

    Ok(())
}

pub async fn zenon(ctx: &Context) -> Result<(), InvocationError> {
    let mut rng = rand::thread_rng();
    ctx.message
        .reply(InputMessage::markdown(format!(
            "dzięki [{name}](tg://user?id={id}) {text}",
            name = ["Zenon", "Zenon Witkowski"].choose(&mut rng).unwrap(),
            id = ZENON,
            text = RESPONSES.choose(&mut rng).unwrap()
        )))
        .await?;

    Ok(())
}

pub async fn zenon_http_noob(ctx: &Context) -> Result<(), InvocationError> {
    ctx.message
        .reply("haha http:// brak szyfrowania noob")
        .await?;

    Ok(())
}

pub async fn polskie_krajobrazy(ctx: &Context) -> Result<(), InvocationError> {
    ctx.message
        .reply(*RESPONSES.choose(&mut rand::thread_rng()).unwrap())
        .await?;

    Ok(())
}

pub async fn say(ctx: &Context) -> Result<(), InvocationError> {
    let mut text = ctx.message.text()[19..].trim();
    if text.to_lowercase().starts_with("@jelnislaw powiedz") {
        text = "haha nob jestes";
    }
    if !text.is_empty() {
        ctx.client.send_message(&ctx.chat, text).await?;
    }

    Ok(())
}
