use async_trait::async_trait;
use grammers_client::client::chats::InvocationError;
use grammers_client::types::{Media, Message};
use grammers_client::Client as GrammersClient;
use grammers_session::PackedChat;
use grammers_tl_types as tl;
use log::{error, warn};

use crate::client::EventHandler;
use crate::commands::{self, Context};
use crate::constants;

pub struct Handler {
    logs_chat: Option<PackedChat>,
}

impl Handler {
    pub fn new() -> Self {
        Self { logs_chat: None }
    }
}

#[async_trait]
impl EventHandler for Handler {
    async fn on_message(
        &self,
        client: GrammersClient,
        message: Message,
    ) -> Result<(), InvocationError> {
        let chat = message.chat();

        let sender_id = message
            .sender()
            .map(|s| s.id())
            .unwrap_or_else(|| chat.id());

        let mut context = Context {
            client: client.clone(),
            chat: message.chat(),
            message,
        };

        match sender_id {
            constants::JELNISLAW => {
                if context.message.text().starts_with('=') {
                    self.invoke_command(&context.message.text()[1..].to_string(), &mut context)
                        .await?;
                }
            }
            constants::JAROSŁAW_KARCEWICZ => {
                if context
                    .message
                    .media()
                    .iter()
                    .any(|m| matches!(m, Media::Document { .. }))
                {
                    commands::strategia(&context).await?;
                }
            }
            constants::ZENON => {
                let text = context.message.text();
                if text.contains("https://") {
                    commands::zenon(&context).await?
                } else if text.contains("http://") {
                    commands::zenon_http_noob(&context).await?;
                }
            }
            constants::POLSKIE_KRAJOBRAZY => {
                if matches!(
                    context.message.forward_header(),
                    Some(tl::enums::MessageFwdHeader::Header(
                        tl::types::MessageFwdHeader {
                            from_id: Some(tl::enums::Peer::Channel(tl::types::PeerChannel {
                                channel_id: constants::POLSKIE_KRAJOBRAZY
                            })),
                            ..
                        }
                    ))
                ) {
                    commands::polskie_krajobrazy(&context).await?;
                }
            }
            _ => (),
        }

        #[allow(clippy::single_match)]
        match chat.id() {
            constants::BAWIALNIA => {
                if context.message.text().starts_with("@JelNiSlaw powiedz ") {
                    commands::say(&context).await?;
                }
            }
            _ => (),
        }

        if context.message.text() == "/prpr@JelNiSlaw" {
            context.message.reply("Peropero").await?;
        }

        Ok(())
    }

    async fn invoke_command(
        &self,
        input: &str,
        context: &mut Context,
    ) -> Result<(), InvocationError> {
        let (command, args) = match input.split_once(' ') {
            Some((command, args)) => (command, Some(args)),
            None => (input, None),
        };

        match (command, args) {
            ("ping", None) => context.message.delete().await?,
            ("id", None) => context.message.edit(context.chat.id().to_string()).await?,
            ("long" | "space", Some(message)) => {
                context
                    .message
                    .edit(message.chars().intersperse(' ').collect::<String>())
                    .await?
            }
            ("zenon", None) => commands::zenon(context).await?,
            ("strategia" | "s", None) => {
                commands::strategia(context).await?;
                context.message.delete().await?
            }
            _ => (),
        };

        Ok(())
    }

    async fn log(&self, client: GrammersClient, message: String) {
        warn!("Logging message: {message:?}");
        let result = client.send_message(self.logs_chat.unwrap(), message).await;
        if let Err(err) = result {
            error!("Couldn't log error: {err}");
        }
    }
}
