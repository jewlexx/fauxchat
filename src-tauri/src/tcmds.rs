use commands::Command;

use crate::ready_message;

#[derive(Debug, thiserror::Error)]
pub enum CommandError {
    #[error("Failed to interact with system IO: {0}")]
    IOError(#[from] std::io::Error),
}

impl serde::Serialize for CommandError {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let error_string = self.to_string();
        serializer.serialize_str(&error_string)
    }
}

type Result<T> = std::result::Result<T, CommandError>;

#[tauri::command]
pub fn invoke_command(command: &str, username: Option<&str>) {
    info!("Invoking command: {}", command);

    let username = username.unwrap_or("random").to_string();

    // TODO: Better error handling
    let parsed = Command::try_from(command.to_string()).expect("valid command");

    ready_message((parsed, username));
}

#[tauri::command]
pub fn load_file(path: &str) -> Result<()> {
    use std::{fs::File, io::Read};

    let file = File::open(path)?;

    Ok(())
}

#[tauri::command]
pub fn send_message(message: &str, username: &str, count: usize, delay: u64) {
    info!("Sending message");

    let command = Command::Send {
        message: message.to_string(),
        count,
        delay,
    };

    ready_message((command, username.to_string()));
}
