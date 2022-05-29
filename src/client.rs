use std::io;
use std::sync::Arc;

use async_trait::async_trait;
use grammers_client::client::chats::{AuthorizationError, InvocationError};
use grammers_client::types::{Dialog, Message, User};
use grammers_client::{Client as GrammersClient, Config, InitParams, SignInError, Update};
use grammers_session::{PackedChat, Session};
use log::{info, warn};

use crate::commands::Context;
use crate::utils::FormatName;
use crate::{constants, utils};

#[async_trait]
pub trait EventHandler: Send + Sync {
    async fn on_message(&self, client: Client, message: Message) -> Result<(), InvocationError>;
    async fn invoke_command(
        &self,
        input: &str,
        context: &mut Context,
    ) -> Result<(), InvocationError>;
    async fn log(&self, client: Client, message: String);
}

#[derive(Clone)]
pub struct Client {
    pub client: GrammersClient,
    event_handler: Arc<dyn EventHandler>,
    session_filename: Arc<str>,
    logs_chat: Option<PackedChat>,
    pub http_client: reqwest::Client,
}

impl Client {
    pub async fn new<H: EventHandler + 'static>(
        event_handler: H,
        session_filename: &str,
    ) -> Result<Self, AuthorizationError> {
        info!("Reading the session file");
        let session = Session::load_file_or_create(session_filename)?;

        Ok(Self {
            client: GrammersClient::connect(Config {
                session,
                api_id: constants::API_ID,
                api_hash: constants::API_HASH.to_string(),
                params: InitParams {
                    app_version: 69.to_string(),
                    catch_up: false,
                    update_queue_limit: None,
                    ..Default::default()
                },
            })
            .await?,
            event_handler: Arc::new(event_handler),
            session_filename: Arc::from(session_filename),
            logs_chat: None,
            http_client: reqwest::Client::new(),
        })
    }

    async fn sign_in(&mut self) -> User {
        let token = loop {
            match self
                .client
                .request_login_code(
                    &utils::prompt_input("Phone: ").unwrap(),
                    constants::API_ID,
                    constants::API_HASH,
                )
                .await
            {
                Ok(token) => break token,
                Err(err) => eprintln!("{}", err),
            }
        };

        let user = loop {
            match self
                .client
                .sign_in(&token, &utils::prompt_input("Code: ").unwrap())
                .await
            {
                Ok(user) => break user,
                Err(SignInError::PasswordRequired(token)) => {
                    match self
                        .client
                        .check_password(token, utils::prompt_password("Password: ").unwrap())
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

        user
    }

    async fn after_login(&mut self) {
        let logs_chat = self
            .get_dialog(constants::LOGS)
            .await
            .unwrap()
            .expect("Could not find logs chat")
            .chat;
        info!("Sending logs to: {}", logs_chat.format_name());
        self.logs_chat = Some(logs_chat.pack());
    }

    pub async fn get_dialog(&self, dialog_id: i64) -> Result<Option<Dialog>, InvocationError> {
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
        self.client.session().save_to_file(&*self.session_filename)
    }

    pub async fn start(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let user = if self.client.is_authorized().await? {
            self.client.get_me().await?
        } else {
            println!("Sign-In required");
            let user = self.sign_in().await;
            self.save_session()?;
            user
        };
        info!("Signed-In as: {}", user.format_name());
        self.after_login().await;
        self.poll_updates().await?;
        warn!("Stopping…");
        self.save_session()?;

        Ok(())
    }

    async fn poll_updates(&self) -> Result<(), Box<dyn std::error::Error>> {
        loop {
            match tokio::select! {
                biased;
                _ = tokio::signal::ctrl_c() => Ok(None),
                result = self.client.next_update() => result,
            } {
                Ok(update) => match update {
                    Some(update) => {
                        let event_handler = self.event_handler.clone();
                        let client = self.clone();
                        tokio::spawn(async move {
                            if let Update::NewMessage(message) = update {
                                if let Err(err) =
                                    event_handler.on_message(client.clone(), message).await
                                {
                                    event_handler
                                        .log(client, format!("Message handler: {err}"))
                                        .await;
                                };
                            }
                        });
                    }
                    None => break,
                },
                Err(err) => {
                    self.event_handler
                        .log(self.clone(), format!("Update loop: {err}"))
                        .await;
                    break;
                }
            };
        }

        Ok(())
    }
}
