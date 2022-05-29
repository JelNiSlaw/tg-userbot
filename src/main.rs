#![feature(iter_intersperse)]

mod client;
mod commands;
mod constants;
mod handler;
mod huggingface;
mod utils;

use client::Client;
use log::LevelFilter;
use simple_logger::SimpleLogger;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    dotenv::dotenv().unwrap();
    SimpleLogger::new()
        .with_level(LevelFilter::Warn)
        .with_module_level("tg_userbot", LevelFilter::Debug)
        .init()
        .unwrap();
    let mut bot = Client::new(handler::Handler::new(), ".session")
        .await
        .unwrap();
    bot.start().await.unwrap();
}
