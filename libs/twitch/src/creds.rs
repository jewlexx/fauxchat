use std::{
    path::PathBuf,
    time::{Duration, SystemTime},
};

use const_format::formatcp;
use once_cell::sync::Lazy;
use parking_lot::Mutex;
use serde::Deserialize;

mod decl;

pub use decl::Credentials;

#[derive(Debug, Deserialize)]
pub struct AccessToken {
    access_token: String,
    refresh_token: String,
}

pub static CREDENTIALS: Lazy<Mutex<Credentials>> = Lazy::new(|| Mutex::new(Credentials::default()));

impl Default for Credentials {
    fn default() -> Self {
        Self::load().unwrap()
    }
}

impl Credentials {
    /// Clone the current credentials, not meant to be modified, but drops the lock
    pub fn read() -> Credentials {
        CREDENTIALS.lock().clone()
    }

    pub async fn init() -> anyhow::Result<()> {
        let mut creds = Credentials::read();

        if creds.remain_30().await? {
            creds.refresh().await?;
        }

        creds.save()?;

        Ok(())
    }

    pub fn load() -> anyhow::Result<Self> {
        use std::{fs::File, io::Read};

        let creds_path = Self::get_path()?;
        if creds_path.exists() {
            let mut file_contents = String::new();

            File::open(creds_path)?.read_to_string(&mut file_contents)?;

            Ok(toml::from_str(&file_contents)?)
        } else {
            let client_id = env!("TWITCH_CLIENT_ID").to_string();
            let client_secret = env!("TWITCH_CLIENT_SECRET").to_string();
            let user_id = env!("TWITCH_USER_ID").to_string();
            let auth_token = env!("TWITCH_AUTH_TOKEN").to_string();
            let refresh_token = env!("TWITCH_REFRESH_TOKEN").to_string();

            let creds = Credentials {
                client_id,
                client_secret,
                user_id,
                auth_token,
                refresh_token,
            };

            creds.save()?;

            Ok(creds)
        }
    }

    pub fn get_path() -> anyhow::Result<PathBuf> {
        use std::fs::create_dir_all;

        let dir = directories::ProjectDirs::from("com", "jewelexx", "FauxChat")
            .unwrap_or_else(|| unimplemented!());

        let data_dir = dir.data_dir();

        if !data_dir.exists() {
            create_dir_all(data_dir)?;
        }

        Ok(data_dir.join("credentials.toml"))
    }

    pub async fn expires_in(&self) -> anyhow::Result<SystemTime> {
        let response: serde_json::Value = reqwest::get("https://id.twitch.tv/oauth2/validate")
            .await?
            .json()
            .await?;

        let expires_in = response["expires_in"].as_u64().ok_or_else(|| {
            anyhow::anyhow!("Could not parse expires_in from response: {:?}", response)
        })?;

        let expires_in_dur = Duration::from_secs(expires_in);

        let now = SystemTime::now() + expires_in_dur;

        Ok(now)
    }

    pub async fn remain_30(&self) -> anyhow::Result<bool> {
        let now = SystemTime::now();
        // Default to now, meaning that it will trigger a refresh
        let expires_in = self.expires_in().await.unwrap_or(now);

        let diff = expires_in.duration_since(now)?;

        Ok(diff < Duration::from_secs(30 * 60))
    }

    pub async fn refresh(&mut self) -> anyhow::Result<()> {
        const CLIENT_ID: &str = env!("TWITCH_CLIENT_ID");
        const CLIENT_SECRET: &str = env!("TWITCH_CLIENT_SECRET");
        const REFRESH_TOKEN: &str = env!("TWITCH_REFRESH_TOKEN");

        const REFRESH_URL: &str = formatcp!(
            "https://id.twitch.tv/oauth2/token?client_id={client_id}&client_secret={client_secret}&grant_type=refresh_token&refresh_token={refresh_token}",
            client_id = CLIENT_ID,
            client_secret = CLIENT_SECRET,
            refresh_token = REFRESH_TOKEN,
        );

        let resp: AccessToken = reqwest::Client::new()
            .post(REFRESH_URL)
            .send()
            .await?
            .json()
            .await?;

        self.auth_token = resp.access_token;

        self.refresh_token = resp.refresh_token;

        self.save()?;

        Ok(())
    }

    pub fn save(&self) -> anyhow::Result<()> {
        use std::{fs::File, io::Write};

        let path = Self::get_path()?;

        let creds_str = toml::to_string(&self)?;

        let mut file = File::create(path)?;

        file.write_all(creds_str.as_bytes())?;

        *CREDENTIALS.lock() = self.clone();

        Ok(())
    }
}
