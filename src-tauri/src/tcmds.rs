use std::{
    fs::File,
    io::{self, BufRead},
    path::Path,
};

use commands::{Command, CommandsError};

use crate::ready_message;

#[derive(Debug, thiserror::Error)]
pub enum CommandError {
    #[error("Failed to interact with system IO: {0}")]
    IOError(#[from] std::io::Error),

    #[error("Failed to parse command: {0}")]
    ParseError(#[from] CommandsError),
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
pub fn invoke_command(command: &str) -> Result<()> {
    info!("Invoking command: {}", command);

    let parsed = Command::try_from(command.to_string())?;

    ready_message(parsed);

    Ok(())
}

#[tauri::command]
pub fn load_file(path: &str) -> Result<()> {
    println!("Loading {path}");
    let lines = read_lines(path)?;

    for line in lines {
        let Ok(parsed) = Command::try_from(line?) else { continue };

        ready_message(parsed);
    }

    Ok(())
}

#[tauri::command]
pub fn send_message(message: &str, username: &str, count: usize, delay: u64) {
    info!("Sending message");

    let command = Command::Send {
        message: message.to_string(),
        username: username.to_string(),
        count,
        delay: commands::amount::Amount::Single(delay),
    };

    ready_message(command);
}

fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where
    P: AsRef<Path>,
{
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}
