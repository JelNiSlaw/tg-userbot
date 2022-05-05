use core::fmt;
use std::fmt::Write as _;
use std::io::{self, Write as _};

use grammers_client::types::User;
pub use rpassword::prompt_password;

pub fn prompt_input(prompt: &str) -> io::Result<String> {
    print!("{}", prompt);
    io::stdout().lock().flush().unwrap();
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    input = input.trim().to_string();
    Ok(input)
}

pub trait DisplayUser {
    fn format_name(&self) -> Result<String, fmt::Error>;
}

impl DisplayUser for User {
    fn format_name(&self) -> Result<String, fmt::Error> {
        let mut name = String::new();
        name.push_str(self.first_name());
        match self.last_name() {
            Some(last_name) => name.write_fmt(format_args!(" {}", last_name))?,
            None => (),
        }
        name.write_fmt(format_args!(
            " ({})",
            match self.username() {
                Some(username) => format!("@{}", username),
                None => self.id().to_string(),
            }
        ))?;

        Ok(name)
    }
}
