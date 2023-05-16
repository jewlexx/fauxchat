// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![warn(clippy::all, clippy::pedantic)]
#![allow(clippy::unsafe_derive_deserialize, clippy::missing_errors_doc)]

use std::path::PathBuf;

use actix_web::{web, App, HttpServer};
use tokio::{fs::File, io::AsyncReadExt};
use tracing_subscriber::fmt::format::FmtSpan;

use faker::twitch_api::{creds::Credentials, TwitchUser};

mod irc;
mod routes;

#[macro_use]
extern crate tracing;

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {name}! You've been greeted from Rust!")
}

#[tauri::command]
fn send_message(message: &str, username: &str, count: usize) {
    let user = {
        if username == "random" {
            TwitchUser::random()
        } else {
            TwitchUser::from_username(username)
        }
    };
    for _ in 0..count {
        faker::MESSAGES
            .lock()
            .push_back((message.to_string(), TwitchUser::from_username(username)));
    }
}

// TODO: In release builds, include all files from chat frontend in binary

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_span_events(FmtSpan::FULL)
        .with_max_level(tracing::Level::INFO)
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

    println!("Created pool");

    *faker::USERS.lock() = pool;

    println!("Assigned users");

    let fut = HttpServer::new(|| {
        println!("Creating app");
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

    println!("Running app");
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![greet, send_message])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
    println!("App closed");

    // Close the server when the app is closed
    server_thread.abort();

    Ok(())
}
