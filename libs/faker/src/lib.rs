#![warn(clippy::all, clippy::pedantic)]
#![allow(clippy::unsafe_derive_deserialize, clippy::missing_errors_doc)]

use std::collections::VecDeque;

use parking_lot::Mutex;

use twitch_api::UserPool;

pub use commands;
pub use twitch_api;
pub use usergen;

pub static USERS: Mutex<UserPool> = Mutex::new(UserPool { users: Vec::new() });

lazy_static::lazy_static! {
    pub static ref MESSAGES: Mutex<VecDeque<String>> = Mutex::new(VecDeque::new());
}
