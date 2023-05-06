#![warn(clippy::all, clippy::pedantic)]
#![allow(clippy::unsafe_derive_deserialize, clippy::missing_errors_doc)]

use tokio::{fs::File, io::AsyncWriteExt};
use twitch_api::UserPool;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    twitch_api::creds::init().await?;

    let pool = UserPool::get().await?;

    let pool_str = serde_json::to_string(&pool)?;

    File::create("pool.json")
        .await?
        .write_all(pool_str.as_bytes())
        .await?;

    Ok(())
}
