#![warn(clippy::all, clippy::pedantic)]
#![allow(clippy::unsafe_derive_deserialize, clippy::missing_errors_doc)]

use std::{num::ParseIntError, time::Duration};

use grammar::CommandsParser;
use thiserror::Error;

#[macro_use]
extern crate pest_derive;

pub mod grammar;

#[derive(Debug, Error)]
pub enum Error {
    #[error("The command provided was invalid. Found {0}")]
    InvalidCommand(String),
    #[error("No command was provided")]
    MissingCommand,
    #[error("No message was provided")]
    MissingMessage,
    #[error("The number provided was invalid")]
    InvalidNumber(#[from] ParseIntError),
    #[error("No number provided")]
    MissingNumber,
}

#[derive(Debug, Copy, Clone)]
pub struct CommandInfo {
    /// A standard command name, in lowercase
    pub name: &'static str,
    /// The number of arguments the command takes
    pub arg_count: usize,
}

impl CommandInfo {
    pub fn from_name(cmd_name: &str) -> Result<CommandInfo, Error> {
        match cmd_name.to_lowercase().as_str() {
            "send" => Ok(CommandInfo {
                name: "send",
                arg_count: 3,
            }),
            "sleep" => Ok(CommandInfo {
                name: "sleep",
                arg_count: 1,
            }),
            _ => Err(Error::InvalidCommand(cmd_name.to_string())),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    /// Sends the given message the given number of times
    Send {
        message: String,
        count: usize,
        delay: u64,
    },
    /// Pauses for the given number of milliseconds
    Sleep { delay: u64 },
}

impl Command {
    pub fn from_parts(parts: &[&str]) -> Result<Command, Error> {
        let cmd_name = parts[0].to_lowercase();
        let cmd_info = CommandInfo::from_name(&cmd_name)?;
        match cmd_info.name {
            "sleep" => Ok(Command::Sleep {
                delay: parts[1].parse()?,
            }),
            "send" => Ok(Command::Send {
                message: {
                    let arg = parts[1].to_string();
                    let lit = litrs::StringLit::parse(arg.as_str()).expect("valid string literal");
                    let value = lit.value();
                    value.to_string()
                },
                count: parts[2].parse()?,
                delay: parts[3].parse()?,
            }),
            _ => unreachable!("Any invalid command error should have been caught above"),
        }
    }

    #[must_use]
    pub fn get_delay(&self) -> Duration {
        let delay_ms = match self {
            Command::Send { delay, .. } | Command::Sleep { delay } => *delay,
        };

        Duration::from_millis(delay_ms)
    }

    #[must_use]
    pub fn get_count(&self) -> usize {
        match self {
            Command::Send { count, .. } => *count,
            Command::Sleep { .. } => 1,
        }
    }
}

impl TryFrom<String> for Command {
    type Error = anyhow::Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        CommandsParser::parse_single(&value)
    }
}
