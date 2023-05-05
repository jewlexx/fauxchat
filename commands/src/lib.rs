#![warn(clippy::all, clippy::pedantic)]
#![allow(clippy::unsafe_derive_deserialize, clippy::missing_errors_doc)]

use std::num::ParseIntError;

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

#[derive(Debug, Clone)]
pub enum Command {
    /// Sends the given message the given number of times
    Send(String, u64),
    /// Pauses for the given number of milliseconds
    Sleep(u64),
}

impl TryFrom<String> for Command {
    type Error = Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        if let Some(command) = value.strip_prefix('/') {
            let split = command.split_whitespace().collect::<Vec<_>>();

            match split.first() {
                Some(&"send") => {
                    let msg = match split.get(1) {
                        Some(v) => Ok(v),
                        None => Err(Self::Error::MissingMessage),
                    }?;

                    let count = match split.get(2) {
                        Some(v) => Ok(v.parse::<u64>()?),
                        _ => Err(Self::Error::MissingNumber),
                    }?;

                    Ok(Command::Send((*msg).to_string(), count))
                }
                Some(&"sleep") => {
                    let count = match split.get(1) {
                        Some(v) => Ok(v.parse::<u64>()?),
                        _ => Err(Self::Error::MissingNumber),
                    }?;

                    Ok(Command::Sleep(count))
                }
                Some(x) => Err(Self::Error::InvalidCommand((*x).to_string())),
                None => Err(Self::Error::MissingCommand),
            }
        } else {
            Ok(Self::Send(value, 1))
        }
    }
}
