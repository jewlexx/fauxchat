// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![warn(clippy::all, clippy::pedantic)]
#![allow(clippy::unsafe_derive_deserialize, clippy::missing_errors_doc)]

use std::path::PathBuf;

use actix_web::{web, App, HttpServer};
use commands::Command;
use crossbeam::channel::{unbounded, Sender};
use once_cell::sync::OnceCell;
use tokio::{fs::File, io::AsyncReadExt};
use tracing_subscriber::fmt::format::FmtSpan;

use twitch_api::creds::Credentials;

mod irc;
mod net;
mod routes;

#[macro_use]
extern crate tracing;

static mut TX: OnceCell<Sender<Command>> = OnceCell::new();

fn ready_message(msg: Command) {
    let tx = unsafe { TX.wait() };

    tx.send(msg).expect("connected channel. receiver dropped?");
}

mod tcmds;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_span_events(FmtSpan::FULL)
        .with_max_level(tracing::Level::DEBUG)
        .init();

    let mut lock = lock::Lock::init()?;
    let guard = lock.try_lock();

    if guard.is_err() {
        eprintln!("Another instance is already running!");
        std::process::exit(1);
    }

    Credentials::init().await?;

    // Must be initialized after credentials
    once_cell::sync::Lazy::force(&twitch_api::CLIENT);

    let pool = if PathBuf::from("../pool.json").exists() {
        let mut file = File::open("../pool.json").await?;
        let mut file_str = String::new();
        file.read_to_string(&mut file_str).await?;
        serde_json::from_str(&file_str)?
    } else {
        twitch_api::UserPool::get().await?
    };

    trace!("Created pool");

    *twitch_api::USERS.lock() = pool;

    trace!("Assigned users");

    let fut = HttpServer::new(|| {
        trace!("Creating app");
        App::new()
            .service(routes::twitch)
            .service(routes::credentials)
            .route("/ws/", web::get().to(irc::handle_ws))
    })
    .bind(net::addr())
    .expect("valid url and successful binding")
    .run();

    let server_thread = tokio::spawn(async move {
        fut.await.expect("valid running of http server");
    });

    let (tx, rx) = unbounded();

    unsafe { TX.set(tx) }.unwrap();

    let messages_thread = tokio::spawn(async move {
        irc::send_messages(&rx);
    });

    trace!("Running app");
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            tcmds::send_message,
            tcmds::invoke_command,
            tcmds::load_file
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
    trace!("App closed");

    // Close the server when the app is closed
    server_thread.abort();
    trace!("Server closed");

    // Drop the sender, thus closing the channel
    unsafe { TX.take() };
    trace!("Dropped sender");
    // Thread will be completed, as we closed the connection
    messages_thread.await?;
    trace!("Messages thread completed");

    Ok(())
}
