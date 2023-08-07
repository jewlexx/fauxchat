#![warn(clippy::all, clippy::pedantic)]
#![allow(clippy::unsafe_derive_deserialize, clippy::missing_errors_doc)]

use std::{num::ParseIntError, time::Duration};

use amount::Amount;
use grammar::{CommandInfo, CommandsParser};
use thiserror::Error;

#[macro_use]
extern crate pest_derive;

pub mod grammar;

pub mod amount;

#[derive(Debug, Error)]
pub enum CommandsError {
    #[error("The number provided was invalid")]
    InvalidNumber(#[from] ParseIntError),
    #[error("Failed to parse grammar: {0}")]
    GrammarError(#[from] grammar::ParseError),
    #[error("Failed to parse Amount value")]
    AmountError(#[from] amount::AmountError),
    #[error("Failed to parse Command from given String. Was given: {0}")]
    ParseCommand(String),
    #[error("No command was provided")]
    MissingCommand,
    #[error("No message was provided")]
    MissingMessage,
    #[error("No number provided")]
    MissingNumber,
}

pub type Result<T> = std::result::Result<T, CommandsError>;

// TODO: Add support for random delay in range
// TODO: i.e rather than 1000, provide 1000..5000, and it picks a random number in that range

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    /// Sends the given message the given number of times
    Send {
        message: String,
        username: String,
        count: usize,
        delay: Amount<u64>,
    },
    /// Pauses for the given number of milliseconds
    Sleep { delay: u64 },
}

impl amount::AmountValue for u64 {}

fn parse_str_lit(lit: &str) -> String {
    let parsed = litrs::StringLit::parse(lit).expect("valid string literal");
    let value = parsed.value();

    value.to_string()
}

impl Command {
    pub fn from_parts(parts: &[&str]) -> Result<Command> {
        let cmd_info = {
            let cmd_name = parts[0].to_lowercase();
            CommandInfo::from_name(&cmd_name)?
        };
        match cmd_info.name {
            "sleep" => Ok(Command::Sleep {
                delay: parts[1].parse()?,
            }),
            "send" => Ok(Command::Send {
                message: parse_str_lit(parts[1]),
                username: parse_str_lit(parts.get(4).copied().unwrap_or("\"random\"")),
                count: parts[2].parse()?,
                delay: dbg!(parts[3]).parse()?,
            }),
            _ => unreachable!("Any invalid command error should have been caught above"),
        }
    }

    #[must_use]
    pub fn get_delay(&self) -> Duration {
        let delay_ms = match self {
            Command::Send { delay, .. } => delay.get_value(),
            Command::Sleep { delay } => *delay,
        };

        Duration::from_millis(delay_ms)
    }
}

impl TryFrom<String> for Command {
    type Error = CommandsError;

    fn try_from(value: String) -> Result<Self> {
        let parts = CommandsParser::parse_parts(&value)?;

        Command::from_parts(&parts)
    }
}

impl std::fmt::Display for Command {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Command::Send {
                message,
                username,
                count,
                delay,
            } => {
                write!(f, "send(\"{message}\", {count}, {delay}")?;

                // Only embed the username if it is not "random"
                if username != "random" {
                    write!(f, ", \"{username}\"")?;
                }

                write!(f, ")")
            }
            Command::Sleep { delay } => {
                write!(f, "sleep({delay})")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parts_to_command() {
        // Test range

        let command = Command::from_parts(&["send", "\"Hello World!\"", "1", "1..10"])
            .expect("successful conversion");
    }

    #[test]
    fn test_command_to_string() {
        let dest = "send(\"Hello, World!\", 3, 1000)";
        let cmd = Command::Send {
            message: String::from("Hello, World!"),
            username: String::from("random"),
            count: 3,
            delay: Amount::Single(1000),
        };

        assert_eq!(cmd.to_string(), dest);

        let dest = "send(\"Hello, World!\", 15, 10, \"justinfan\")";
        let cmd = Command::Send {
            message: String::from("Hello, World!"),
            username: String::from("justinfan"),
            count: 15,
            delay: Amount::Single(10),
        };

        assert_eq!(cmd.to_string(), dest);
    }
}
