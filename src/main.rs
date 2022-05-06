mod utils;

use std::{error, io};

use grammers_client::client::chats::AuthorizationError;
use grammers_client::types::{Chat, Message, User};
use grammers_client::{Client, Config, InitParams, InputMessage, SignInError, Update};
use grammers_session::Session;
use rand::seq::SliceRandom;

use crate::utils::DisplayUser;

const API_ID: i32 = 15608824;
const API_HASH: &str = "234be898e0230563009e9e12d8a2e546";

const JELNISLAW: i64 = 807128293;
const BAWIALNIA: i64 = 1463139920;
const ZENON: i64 = 2125785292;

const RESPONSES: [&str; 4] = [
    "zamknij ryj",
    "bądź cicho",
    "przestań spamić",
    "super materiał (nie)",
];

struct Bot {
    pub client: Client,
    session_filename: String,
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
            session_filename: session_filename.to_string(),
        })
    }

    async fn login(&mut self) -> Result<User, Box<dyn error::Error>> {
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

    async fn poll_updates(&mut self) -> Result<(), Box<dyn error::Error>> {
        loop {
            match self.client.next_update().await? {
                Some(update) => {
                    if let Update::NewMessage(message) = update {
                        self.on_message(message).await?;
                    }
                }
                None => break,
            }
        }

        Ok(())
    }

    async fn on_message(&self, message: Message) -> Result<(), Box<dyn error::Error>> {
        if let Some(Chat::User(sender)) = message.sender() {
            println!("{}: {:?}", sender.format_name()?, message.text());

            if sender.id() == ZENON && message.text().contains("https://youtu.be/") {
                let mut text = String::from("dzięki Zenon ");
                text.push_str(RESPONSES.choose(&mut rand::thread_rng()).unwrap());
                println!("{}", text);
                self.client
                    .send_message(
                        &message.chat(),
                        InputMessage::text(text).reply_to(Some(message.id())),
                    )
                    .await?;
            }

            if message.chat().id() == BAWIALNIA && message.text().starts_with("@JelNiSlaw powiedz ")
            {
                let mut text = message.text()[19..].trim();
                if text.to_lowercase().starts_with("@jelnislaw powiedz") {
                    text = "haha nob jestes";
                }
                if !text.is_empty() {
                    self.client.send_message(message.chat(), text).await?;
                }
            }
        }

        Ok(())
    }

    fn save_session(&self) -> io::Result<()> {
        self.client.session().save_to_file(&self.session_filename)
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn error::Error>> {
    let mut bot = Bot::new(".session").await?;

    let user = if bot.client.is_authorized().await? {
        bot.client.get_me().await?
    } else {
        println!("Sign-In required");
        let user = bot.login().await?;
        bot.save_session()?;
        user
    };

    println!("Signed-In as: {}", user.format_name()?);

    bot.poll_updates().await?;

    Ok(())
}
