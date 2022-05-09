use grammers_client::client::chats::InvocationError;
use grammers_client::types::{Chat, Message};
use grammers_client::{Client, InputMessage};
use rand::prelude::SliceRandom;
use rand::Rng;

use crate::{RESPONSES, ZENON};

pub struct Context<'m> {
    pub client: Client,
    pub message: &'m mut Message,
    pub chat: Chat,
}

pub async fn strategia(ctx: &Context<'_>) -> Result<(), InvocationError> {
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

pub async fn zenon(ctx: &Context<'_>) -> Result<(), InvocationError> {
    ctx.message
        .reply(
            InputMessage::markdown(format!(
                "dzięki [Zenon](tg://user?id={}) {}",
                ZENON,
                RESPONSES.choose(&mut rand::thread_rng()).unwrap()
            ))
            .reply_to(Some(ctx.message.id())),
        )
        .await?;

    Ok(())
}

pub async fn polskie_krajobrazy(ctx: &Context<'_>) -> Result<(), InvocationError> {
    ctx.message
        .reply(*RESPONSES.choose(&mut rand::thread_rng()).unwrap())
        .await?;

    Ok(())
}

pub async fn say(ctx: &Context<'_>) -> Result<(), InvocationError> {
    let mut text = ctx.message.text()[19..].trim();
    if text.to_lowercase().starts_with("@jelnislaw powiedz") {
        text = "haha nob jestes";
    }
    if !text.is_empty() {
        ctx.client.send_message(&ctx.chat, text).await?;
    }

    Ok(())
}
