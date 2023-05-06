// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![warn(clippy::all, clippy::pedantic)]
#![allow(clippy::unsafe_derive_deserialize, clippy::missing_errors_doc)]

use std::{collections::VecDeque, io::Read, path::PathBuf};

use actix_web::{web, App, HttpServer};
use clap::Parser;
use tokio::{fs::File, io::AsyncReadExt};
use tracing_subscriber::fmt::format::FmtSpan;

use faker::twitch_api::creds::Credentials;

mod irc;
mod routes;

#[macro_use]
extern crate tracing;

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {name}! You've been greeted from Rust!")
}

// TODO: In release builds, include all files from chat frontend in binary

#[derive(Debug, Parser)]
struct CmdArgs {
    #[clap(short, long)]
    messages_file: Option<PathBuf>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_span_events(FmtSpan::FULL)
        .with_max_level(tracing::Level::INFO)
        .init();

    let args = CmdArgs::parse();

    tokio::spawn(async {
        loop {
            use std::io;

            let mut buf = String::new();

            if io::stdin().read_line(&mut buf).is_ok() {
                faker::MESSAGES.lock().push_back(buf);
            }
        }
    });

    Credentials::init().await?;

    // Must be initialized after credentials
    lazy_static::initialize(&faker::twitch_api::CLIENT);

    {
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

        // A file containing one message per line
        let msgs_path = {
            let cwd = std::env::current_dir().unwrap();

            if let Some(path) = args.messages_file {
                cwd.join(path)
            } else {
                cwd.join("messages.txt")
            }
        };

        println!("Opening messages");
        let mut msgs_file = File::open(msgs_path).await?;

        println!("Created messages string");
        let mut msgs_str = String::new();

        println!("Reading messages");
        msgs_file.read_to_string(&mut msgs_str).await?;

        println!("Parsing messages");
        let msgs: VecDeque<String> = msgs_str.lines().map(String::from).collect();

        println!("Assigning messages");
        *faker::MESSAGES.lock() = msgs;
        println!("Assigned messages");
    }

    println!("Running app");
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![greet])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
    println!("Ran app");

    HttpServer::new(|| {
        App::new()
            .service(routes::twitch)
            .service(routes::credentials)
            .route("/ws/", web::get().to(irc::handle_ws))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await?;

    Ok(())
}
