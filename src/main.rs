#![feature(iter_intersperse)]

mod commands;
mod utils;

use std::io;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use commands::Context;
use grammers_client::client::chats::{AuthorizationError, InvocationError};
use grammers_client::types::{Dialog, Media, Message, User};
use grammers_client::{Client, Config, InitParams, SignInError, Update};
use grammers_session::{PackedChat, Session};
use grammers_tl_types as tl;
use log::{error, info, warn, LevelFilter};
use simple_logger::SimpleLogger;

use crate::utils::FormatName;

const API_ID: i32 = 15608824;
const API_HASH: &str = "234be898e0230563009e9e12d8a2e546";

const JELNISLAW: i64 = 807128293;
const LOGS: i64 = 1714064879;
const BAWIALNIA: i64 = 1463139920;
const JAROSŁAW_KARCEWICZ: i64 = 2128162985;
const ZENON: i64 = 2125785292;
const POLSKIE_KRAJOBRAZY: i64 = 1408357156;

struct Bot {
    pub client: Client,
    running: Arc<AtomicBool>,
    session_filename: String,
    logs_chat: Option<PackedChat>,
}

impl Bot {
    async fn new(session_filename: &str) -> Result<Self, AuthorizationError> {
        info!("Reading the session file");
        let session = Session::load_file_or_create(session_filename)?;
        Ok(Self {
            client: Client::connect(Config {
                session,
                api_id: API_ID,
                api_hash: API_HASH.to_string(),
                params: InitParams {
                    app_version: 69.to_string(),
                    catch_up: false,
                    update_queue_limit: None,
                    ..Default::default()
                },
            })
            .await?,
            running: Arc::new(AtomicBool::new(true)),
            session_filename: session_filename.to_string(),
            logs_chat: None,
        })
    }

    async fn sign_in(&mut self) -> Result<User, Box<dyn std::error::Error>> {
        let token = loop {
            match self
                .client
                .request_login_code(&utils::prompt_input("Phone: ")?, API_ID, API_HASH)
                .await
            {
                Ok(token) => break token,
                Err(err) => eprintln!("{}", err),
            }
        };

        let user = loop {
            match self
                .client
                .sign_in(&token, &utils::prompt_input("Code: ")?)
                .await
            {
                Ok(user) => break user,
                Err(SignInError::PasswordRequired(token)) => {
                    match self
                        .client
                        .check_password(token, utils::prompt_password("Password: ")?)
                        .await
                    {
                        Ok(user) => break user,
                        Err(err) => {
                            eprintln!("{}", err);
                            continue;
                        }
                    }
                }
                Err(err) => eprintln!("{}", err),
            }
        };

        Ok(user)
    }

    async fn after_login(&mut self) -> Result<(), InvocationError> {
        let logs_chat = self
            .get_dialog(LOGS)
            .await?
            .expect("Could not find logs chat")
            .chat;
        info!("Sending logs to: {}", logs_chat.format_name());
        self.logs_chat = Some(logs_chat.pack());

        Ok(())
    }

    async fn get_dialog(&self, dialog_id: i64) -> Result<Option<Dialog>, InvocationError> {
        let mut dialogs = self.client.iter_dialogs();

        while let Some(dialog) = dialogs.next().await? {
            if dialog.chat.id() == dialog_id {
                return Ok(Some(dialog));
            }
        }

        Ok(None)
    }

    fn save_session(&self) -> io::Result<()> {
        info!("Saving the session file");
        self.client.session().save_to_file(&self.session_filename)
    }

    async fn start(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let user = if self.client.is_authorized().await? {
            self.client.get_me().await?
        } else {
            println!("Sign-In required");
            let user = self.sign_in().await?;
            self.save_session()?;
            user
        };
        info!("Signed-In as: {}", user.format_name());
        self.after_login().await?;
        let running = self.running.clone();
        tokio::spawn(async move {
            tokio::signal::ctrl_c().await.unwrap();
            warn!("Stopping…");
            running.store(false, Ordering::Relaxed);
        });
        self.poll_updates().await?;
        self.save_session()?;

        Ok(())
    }

    async fn poll_updates(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        while self.running.load(Ordering::Relaxed) {
            match self.client.next_update().await {
                Ok(update) => match update {
                    Some(update) => {
                        if let Update::NewMessage(message) = update {
                            if let Err(err) = self.on_message(message).await {
                                self.log(&format!("Message handler: {err}")).await?;
                            }
                        }
                    }
                    None => return Ok(()),
                },
                Err(err) => {
                    if let Err(err) = self.log(&format!("Update loop: {err}")).await {
                        error!("Logging error: {err}");
                    }
                }
            };
        }

        Ok(())
    }

    async fn on_message(&mut self, message: Message) -> Result<(), Box<dyn std::error::Error>> {
        let chat = message.chat();

        let sender_id = message
            .sender()
            .map(|s| s.id())
            .unwrap_or_else(|| chat.id());

        let mut context = Context {
            client: self.client.clone(),
            chat: message.chat(),
            message,
        };

        match sender_id {
            JELNISLAW => {
                if context.message.text().starts_with('=') {
                    self.invoke_command(&context.message.text()[1..].to_string(), &mut context)
                        .await?;
                }
            }
            JAROSŁAW_KARCEWICZ => {
                if context
                    .message
                    .media()
                    .iter()
                    .any(|m| matches!(m, Media::Document { .. }))
                {
                    commands::strategia(&context).await?;
                }
            }
            ZENON => {
                let text = context.message.text();
                if text.contains("https://") {
                    commands::zenon(&context).await?
                } else if text.contains("http://") {
                    commands::zenon_http_noob(&context).await?;
                }
            }
            POLSKIE_KRAJOBRAZY => {
                if matches!(
                    context.message.forward_header(),
                    Some(tl::enums::MessageFwdHeader::Header(
                        tl::types::MessageFwdHeader {
                            from_id: Some(tl::enums::Peer::Channel(tl::types::PeerChannel {
                                channel_id: POLSKIE_KRAJOBRAZY
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
            BAWIALNIA => {
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
        &mut self,
        input: &str,
        context: &mut Context,
    ) -> Result<(), InvocationError> {
        let (command, args) = match input.split_once(' ') {
            Some((command, args)) => (command, Some(args)),
            None => (input, None),
        };

        match (command, args) {
            ("ping", None) => context.message.delete().await?,
            ("stop", None) => {
                warn!("Stopping…");
                self.running.store(false, Ordering::Relaxed);
                context.message.delete().await?;
            }
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

    async fn log(&self, message: &str) -> Result<(), InvocationError> {
        warn!("Logging message: {message:?}");
        self.client
            .send_message(self.logs_chat.unwrap(), message)
            .await?;

        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    SimpleLogger::new()
        .with_level(LevelFilter::Warn)
        .with_module_level("tg_userbot", LevelFilter::Debug)
        .init()?;
    let mut bot = Bot::new(".session").await?;
    bot.start().await
}
