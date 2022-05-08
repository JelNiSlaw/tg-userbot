#![feature(iter_intersperse)]

mod utils;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::{error, io};

use grammers_client::client::chats::{AuthorizationError, InvocationError};
use grammers_client::types::{Chat, Media, Message, User};
use grammers_client::{Client, Config, InitParams, InputMessage, SignInError, Update};
use grammers_session::{PackedChat, Session};
use grammers_tl_types as tl;
use rand::seq::SliceRandom;
use rand::Rng;

use crate::utils::FormatName;

const API_ID: i32 = 15608824;
const API_HASH: &str = "234be898e0230563009e9e12d8a2e546";

const JELNISLAW: i64 = 807128293;
const LOGS: i64 = 1714064879;
const BAWIALNIA: i64 = 1463139920;
const JAROSŁAW_KARCEWICZ: i64 = 2128162985;
const ZENON: i64 = 2125785292;
const POLSKIE_KRAJOBRAZY: i64 = 1408357156;

const RESPONSES: [&str; 5] = [
    "zamknij ryj",
    "bądź cicho",
    "cicho bądź",
    "przestań spamić",
    "super materiał (nie)",
];

struct Bot {
    pub client: Client,
    running: Arc<AtomicBool>,
    session_filename: String,
    logs_chat: Option<PackedChat>,
}

impl Bot {
    async fn new(session_filename: &str) -> Result<Self, AuthorizationError> {
        println!("Reading the session file");
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

    async fn sign_in(&mut self) -> Result<User, Box<dyn error::Error>> {
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
            .get_chat(LOGS)
            .await?
            .expect("Could not find logs chat");
        println!("Sending logs to: {}", logs_chat.format_name());
        self.logs_chat = Some(logs_chat.pack());

        Ok(())
    }

    async fn get_chat(&self, chat_id: i64) -> Result<Option<Chat>, InvocationError> {
        let mut dialogs = self.client.iter_dialogs();
        loop {
            match dialogs.next().await? {
                Some(dialog) => {
                    if dialog.chat.id() == chat_id {
                        return Ok(Some(dialog.chat));
                    }
                }
                None => return Ok(None),
            }
        }
    }

    fn save_session(&self) -> io::Result<()> {
        self.client.session().save_to_file(&self.session_filename)
    }

    async fn start(&mut self) -> Result<(), Box<dyn error::Error>> {
        let user = if self.client.is_authorized().await? {
            self.client.get_me().await?
        } else {
            println!("Sign-In required");
            let user = self.sign_in().await?;
            self.save_session()?;
            user
        };
        println!("Signed-In as: {}", user.format_name());
        self.after_login().await?;
        let running = self.running.clone();
        tokio::spawn(async move {
            tokio::signal::ctrl_c().await.unwrap();
            println!("Stopping…");
            running.store(false, Ordering::Relaxed);
        });
        self.poll_updates().await
    }

    async fn poll_updates(&mut self) -> Result<(), Box<dyn error::Error>> {
        while self.running.load(Ordering::Relaxed) {
            match self.client.next_update().await {
                Ok(update) => match update {
                    Some(update) => {
                        if let Update::NewMessage(mut message) = update {
                            if let Err(err) = self.on_message(&mut message).await {
                                self.log(&format!("Message handler: {err}")).await?;
                            }
                        }
                    }
                    None => return Ok(()),
                },
                Err(err) => self.log(&format!("Update loop: {err}")).await?,
            }
        }

        Ok(())
    }

    async fn on_message(&mut self, message: &mut Message) -> Result<(), Box<dyn error::Error>> {
        let chat = message.chat();

        let sender_id = message
            .sender()
            .map(|s| s.id())
            .unwrap_or_else(|| chat.id());

        match sender_id {
            JELNISLAW => {
                if message.text().starts_with('=') {
                    self.invoke_command(&message.text()[1..].to_string(), message)
                        .await?;
                }
            }
            JAROSŁAW_KARCEWICZ => {
                if message
                    .media()
                    .iter()
                    .any(|m| matches!(m, Media::Document { .. }))
                {
                    self.strategia(&chat).await?;
                }
            }
            ZENON => {
                if message.text().contains("https://youtu.be/") {
                    self.zenon(message).await?
                }
            }
            POLSKIE_KRAJOBRAZY => {
                if matches!(
                    message.forward_header(),
                    Some(tl::enums::MessageFwdHeader::Header(
                        tl::types::MessageFwdHeader {
                            from_id: Some(tl::enums::Peer::Channel(tl::types::PeerChannel {
                                channel_id: POLSKIE_KRAJOBRAZY
                            })),
                            ..
                        }
                    ))
                ) {
                    self.polskie_krajobrazy(message).await?;
                }
            }
            _ => (),
        }

        #[allow(clippy::single_match)]
        match chat.id() {
            BAWIALNIA => {
                if message.text().starts_with("@JelNiSlaw powiedz ") {
                    self.say(message, &chat).await?;
                }
            }
            _ => (),
        }

        Ok(())
    }

    async fn invoke_command(
        &mut self,
        input: &str,
        message: &mut Message,
    ) -> Result<(), InvocationError> {
        let (command, args) = match input.split_once(' ') {
            Some((command, args)) => (command, Some(args)),
            None => (input, None),
        };

        match (command, args) {
            ("ping", None) => message.delete().await?,
            ("stop", None) => {
                self.running.store(false, Ordering::Relaxed);
                message.delete().await?;
            }
            ("id", None) => message.edit(message.chat().id().to_string()).await?,
            ("long" | "space", Some(text)) => {
                message
                    .edit(text.chars().intersperse(' ').collect::<String>())
                    .await?
            }
            ("strategia" | "s", None) => {
                self.strategia(&message.chat()).await?;
                message.delete().await?
            }
            _ => (),
        };

        Ok(())
    }

    async fn log(&self, text: &str) -> Result<(), InvocationError> {
        println!("Logging message: {text:?}");
        self.client
            .send_message(self.logs_chat.unwrap(), text)
            .await?;

        Ok(())
    }

    async fn strategia(&self, chat: &Chat) -> Result<(), InvocationError> {
        let mut rng = rand::thread_rng();
        let mut messages = Vec::new();
        while rng.gen::<bool>() {
            messages.push(
                self.client
                    .send_message(
                        chat,
                        *["strategia", "strateg", "strategicznie"]
                            .choose(&mut rng)
                            .unwrap(),
                    )
                    .await?
                    .id(),
            );
        }
        self.client.delete_messages(chat, &messages).await?;

        Ok(())
    }

    async fn zenon(&self, message: &Message) -> Result<(), InvocationError> {
        let mut text = String::from("dzięki Zenon ");
        text.push_str(RESPONSES.choose(&mut rand::thread_rng()).unwrap());
        println!("{}", text);
        message
            .reply(InputMessage::text(text).reply_to(Some(message.id())))
            .await?;

        Ok(())
    }

    async fn polskie_krajobrazy(&self, message: &Message) -> Result<(), InvocationError> {
        message
            .reply(*RESPONSES.choose(&mut rand::thread_rng()).unwrap())
            .await?;

        Ok(())
    }

    async fn say(&self, message: &Message, chat: &Chat) -> Result<(), InvocationError> {
        let mut text = message.text()[19..].trim();
        if text.to_lowercase().starts_with("@jelnislaw powiedz") {
            text = "haha nob jestes";
        }
        if !text.is_empty() {
            self.client.send_message(chat, text).await?;
        }

        Ok(())
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn error::Error>> {
    let mut bot = Bot::new(".session").await?;
    bot.start().await
}
