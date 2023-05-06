#![warn(clippy::all, clippy::pedantic)]
#![allow(clippy::unsafe_derive_deserialize, clippy::missing_errors_doc)]

use std::collections::VecDeque;

use tokio::sync::Mutex;
use twitch_api::UserPool;

pub use twitch_api;

pub mod irc;

#[macro_use]
extern crate tracing;

pub static USERS: Mutex<UserPool> = Mutex::const_new(UserPool { users: Vec::new() });

lazy_static::lazy_static! {
    pub static ref MESSAGES: Mutex<VecDeque<String>> = Mutex::new(VecDeque::new());
}
