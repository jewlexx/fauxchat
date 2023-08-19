// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![warn(clippy::all, clippy::pedantic)]
#![allow(clippy::unsafe_derive_deserialize, clippy::missing_errors_doc)]

use std::{path::PathBuf, sync::Arc};

use actix_web::{web, App, HttpServer};
use commands::Command;
use crossbeam::channel::{unbounded, Sender};
use once_cell::sync::OnceCell;
use time::macros::format_description;
use tokio::{fs::File, io::AsyncReadExt};
use tracing_subscriber::fmt::format::FmtSpan;

use twitch_api::creds::Credentials;

mod irc;
mod net;
mod routes;

#[macro_use]
extern crate tracing;

static mut TX: OnceCell<Sender<Command>> = OnceCell::new();

// #[cfg(not(debug_assertions))]
fn cmdir_dir() -> PathBuf {
    directories::ProjectDirs::from("com", "jewelexx", "FauxChat")
        .unwrap()
        .cache_dir()
        .to_path_buf()
}

// #[cfg(debug_assertions)]
// fn cmdir_dir() -> PathBuf {
//     PathBuf::new()
// }

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
    let guard = Arc::new(lock.try_lock());

    if guard.is_err() {
        #[cfg(not(debug_assertions))]
        tauri::api::dialog::blocking::message::<tauri::Wry>(
            None,
            "Already Running!",
            "Another instance is already running! Close it before running FauxChat again.",
        );

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

    let cmdir_path = {
        let folder = cmdir_dir();

        std::fs::create_dir_all(&folder).expect("created cmdir directory");

        let file_name = {
            let now: time::OffsetDateTime = std::time::SystemTime::now().into();

            let formatted_date = now
                .format(format_description!(
                    "[year]-[month]-[day]-[hour]-[minute]-[second]"
                ))
                .unwrap();

            // Save as .cmdir file (short for command intermediate representation)
            // This file will require parsing to have the "end_pause" converted into regular sleep commands
            formatted_date + ".cmdir"
        };

        folder.join(file_name)
    };

    let messages_thread = {
        let path = cmdir_path.clone();
        tokio::spawn(async move {
            irc::send_messages(&rx, path);
        })
    };

    // TODO: Parse cmdir into .commands

    trace!("Running app");
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            tcmds::send_message,
            tcmds::invoke_command,
            tcmds::load_file
        ])
        // .setup(|app| {
        //     let window = app.get_window("main").unwrap();
        //     Ok(())
        // })
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

    // TODO: Parse cmdir into regular cmd file

    Ok(())
}
