// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![warn(clippy::all, clippy::pedantic)]
#![allow(clippy::unsafe_derive_deserialize, clippy::missing_errors_doc)]

use std::{path::PathBuf, time::Duration};

use actix_web::{web, App, HttpServer};
use tokio::{fs::File, io::AsyncReadExt};
use tracing_subscriber::fmt::format::FmtSpan;

use faker::{
    commands::{self, Command},
    twitch_api::{creds::Credentials, TwitchUser},
    MESSAGES,
};

mod irc;
mod routes;

#[macro_use]
extern crate tracing;

#[tauri::command]
fn invoke_command(command: &str, username: Option<&str>) {
    info!("Invoking command: {}", command);

    let username = username.unwrap_or("random").to_string();

    let parsed = Command::try_from(command.to_string()).expect("valid command");

    let mut messages = faker::MESSAGES.lock();

    messages.push_back((parsed, username));
}

#[tauri::command]
fn send_message(message: &str, username: &str, count: usize, delay: u64) {
    println!("Sending message");

    let command = commands::Command::Send {
        message: message.to_string(),
        count,
        delay,
    };

    faker::MESSAGES
        .lock()
        .push_back((command, username.to_string()));
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_span_events(FmtSpan::FULL)
        .with_max_level(tracing::Level::DEBUG)
        .init();

    Credentials::init().await?;

    // Must be initialized after credentials
    once_cell::sync::Lazy::force(&faker::twitch_api::CLIENT);

    let pool = if PathBuf::from("../pool.json").exists() {
        let mut file = File::open("../pool.json").await?;
        let mut file_str = String::new();
        file.read_to_string(&mut file_str).await?;
        serde_json::from_str(&file_str)?
    } else {
        faker::twitch_api::UserPool::get().await?
    };

    trace!("Created pool");

    *faker::USERS.lock() = pool;

    trace!("Assigned users");

    let fut = HttpServer::new(|| {
        trace!("Creating app");
        App::new()
            .service(routes::twitch)
            .service(routes::credentials)
            .route("/ws/", web::get().to(irc::handle_ws))
    })
    .bind(faker::addr())
    .expect("valid url and successful binding")
    .run();

    let server_thread = tokio::spawn(async move {
        fut.await.expect("valid running of http server");
    });

    let messages_thread = tokio::spawn(async {
        let mut interval = tokio::time::interval(Duration::from_millis(1000));

        loop {
            interval.tick().await;
            irc::send_messages();
        }
    });

    trace!("Running app");
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![send_message, invoke_command])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
    trace!("App closed");

    // Close the server when the app is closed
    server_thread.abort();
    messages_thread.abort();

    Ok(())
}
