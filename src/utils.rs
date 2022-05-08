use std::fmt::Write as _;
use std::io::{self, Write as _};

use grammers_client::types::{Channel, Chat, Group, User};
pub use rpassword::prompt_password;

pub fn prompt_input(prompt: &str) -> io::Result<String> {
    print!("{}", prompt);
    io::stdout().lock().flush().unwrap();
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    input = input.trim().to_string();
    Ok(input)
}

pub trait FormatName {
    fn format_name(&self) -> String;
}

impl FormatName for User {
    fn format_name(&self) -> String {
        let mut name = self.full_name();

        let username = match self.username() {
            Some(username) => format!("@{}", username),
            None => format!("{}", self.id()),
        };

        match name.is_empty() {
            false => {
                name.write_fmt(format_args!(" ({})", username)).unwrap();
                name
            }
            true => username,
        }
    }
}

impl FormatName for Group {
    fn format_name(&self) -> String {
        format!("{} ({})", self.title(), self.id())
    }
}

impl FormatName for Channel {
    fn format_name(&self) -> String {
        format!("{} ({})", self.title(), self.id())
    }
}

impl FormatName for Chat {
    fn format_name(&self) -> String {
        format!("{} ({})", self.name(), self.id())
    }
}
