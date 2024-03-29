#![warn(clippy::all, clippy::pedantic)]
#![allow(clippy::unsafe_derive_deserialize, clippy::missing_errors_doc)]

use once_cell::sync::Lazy;
use parking_lot::Mutex;
use rand::seq::SliceRandom;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use usergen::Color;

pub mod creds;

pub static USERS: Mutex<UserPool> = Mutex::new(UserPool { users: Vec::new() });

#[macro_export]
macro_rules! api_url {
    ($url:literal) => {
        format!(
            "https://api.twitch.tv/helix/{}",
            format!($url, user_id = env!("TWITCH_USER_ID"))
        )
    };
}

pub static CLIENT: Lazy<reqwest::Client> = Lazy::new(|| {
    use reqwest::header::HeaderValue;
    let mut default_headers = reqwest::header::HeaderMap::new();
    default_headers.insert(
        "Client-Id",
        HeaderValue::from_str(&crate::creds::CREDENTIALS.lock().client_id).unwrap(),
    );
    default_headers.insert(
        "Authorization",
        HeaderValue::from_str(&format!(
            "Bearer {}",
            crate::creds::CREDENTIALS.lock().auth_token
        ))
        .unwrap(),
    );

    reqwest::Client::builder()
        .default_headers(default_headers)
        .build()
        .unwrap()
});

// Must retrieve list of followers, subscribers, mods, vips, etc. and match against the list of users in the channel

#[derive(Debug, Serialize, Deserialize)]
pub struct TwitchVips {
    pub data: Vec<VipDatum>,
    pub pagination: Pagination,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VipDatum {
    pub user_id: String,
    pub user_name: String,
    pub user_login: String,
}

impl TwitchVips {
    async fn from_api(url: String) -> anyhow::Result<Self> {
        let mut data: TwitchVips = CLIENT.get(&url).send().await?.json().await?;

        while let Some(ref cursor) = data.pagination.cursor {
            let url = format!("{url}&after={cursor}");
            let new_data: TwitchVips = {
                let txt = CLIENT.get(dbg!(url)).send().await?.text().await?;
                serde_json::from_str(&dbg!(txt))?
            };

            data.data.extend(new_data.data);
            data.pagination = new_data.pagination;
        }

        Ok(data)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserPool {
    pub users: Vec<TwitchUser>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TwitchUser {
    pub name: String,
    pub uid: String,
    pub color: Color,
    pub is_mod: bool,
    pub is_vip: bool,
    pub is_sub: bool,
}

impl TwitchUser {
    /// # Panics
    /// - If the list of users is empty (which it should never be)
    pub fn random() -> Self {
        let mut rng = rand::thread_rng();
        let users = &crate::USERS.lock().users;
        users.choose(&mut rng).unwrap().clone()
    }

    pub fn from_username(username: impl AsRef<str>) -> Self {
        let username = username.as_ref();

        crate::USERS
            .lock()
            .users
            .iter()
            .find(|user| user.name == username)
            .cloned()
            .unwrap_or_else(|| Self::fake_from_username(username))
    }

    pub fn fake_from_username(username: impl AsRef<str>) -> Self {
        use rand::Rng;

        let mut rng = rand::thread_rng();

        Self {
            name: username.as_ref().to_string(),
            uid: "fake_uid".to_string(),
            color: Color::generate_light(),
            is_mod: false,
            is_vip: false,
            is_sub: rng.gen(),
        }
    }

    // pub fn randomize_properties(&mut self) {
    //     use rand::Rng;

    //     let mut rng = rand::thread_rng();

    //     self.is_sub = rng.gen_range(1..1000) < 200;
    //     self.is_vip = rng.gen_range(1..1000) < 50;
    //     self.is_mod = rng.gen_range(1..1000) < ;
    // }
}

pub struct Badges {
    inner: Vec<Badge>,
}

impl Badges {
    #[must_use]
    pub fn from_user(user: &TwitchUser) -> Self {
        let uid = &crate::creds::CREDENTIALS.lock().user_id;

        let mut badges = Vec::new();

        if uid == &user.uid {
            badges.push(Badge::Broadcaster);
        }

        if user.is_mod {
            badges.push(Badge::Moderator);
        }

        if user.is_vip {
            badges.push(Badge::Vip);
        }

        if user.is_sub {
            badges.push(Badge::Subscriber);
        }

        Self { inner: badges }
    }
}

pub enum Badge {
    Broadcaster,
    Subscriber,
    Moderator,
    Vip,
}

impl std::fmt::Display for Badge {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Broadcaster => write!(f, "broadcaster/1"),
            Self::Subscriber => write!(f, "subscriber/3012"),
            Self::Vip => write!(f, "vip/1"),
            Self::Moderator => write!(f, "moderator/1"),
        }
    }
}

impl std::fmt::Display for Badges {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "badges=")?;

        for (i, badge) in self.inner.iter().enumerate() {
            write!(f, "{badge}")?;

            if i != self.inner.len() - 1 {
                write!(f, ",")?;
            }
        }

        write!(f, ";")?;

        Ok(())
    }
}

impl TwitchUser {
    pub fn send_message(&self, message: impl AsRef<str>) -> String {
        let msg = message.as_ref();

        let badges = Badges::from_user(self);

        let mut message = format!(
            "@badge-info={};",
            if self.is_sub { "subscriber/22" } else { "" }
        );

        message.push_str(badges.to_string().as_str());

        message.push_str("client-nonce=6090b7621f1bf7bdcc46777cd522bca1;");

        message.push_str(&format!("color=#{:X};", self.color));

        message.push_str(&format!("display-name={};", self.name));

        message.push_str("emotes=;first-msg=0;flags=;id=aedfa462-66b6-4a2b-b94d-afb01d0631f9;");

        message.push_str(&format!("mod={};", if self.is_mod { "1" } else { "0" }));

        message.push_str("returning-chatter=0;");

        message.push_str(const_format::concatcp!(
            "room-id=",
            env!("TWITCH_USER_ID"),
            ";"
        ));

        message.push_str(&format!(
            "subscriber={};",
            if self.is_sub { "1" } else { "0" }
        ));

        let current_time = {
            use std::time::{SystemTime, UNIX_EPOCH};

            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("Time went backwards")
                .as_millis()
        };

        message.push_str(&format!("tmi-sent-ts={current_time};"));

        message.push_str("turbo=0;");

        message.push_str(&format!("user-id={};", self.uid));

        message.push_str("user-type= :");

        message.push_str(&format!(
            "{}!{}@{}.tmi.twitch.tv PRIVMSG #{} :{}",
            self.name, self.name, self.name, self.name, msg
        ));

        message
    }
}

impl UserPool {
    pub async fn get() -> anyhow::Result<Self> {
        println!("Downloading pool");
        println!("Downloading pool");
        println!("Downloading pool");
        println!("Downloading pool");
        println!("Downloading pool");
        println!("Downloading pool");
        println!("Downloading pool");

        let vips = TwitchVips::from_api(crate::api_url!(
            "channels/vips?broadcaster_id={user_id}&first=100"
        ))
        .await?;

        let mods = TwitchVips::from_api(crate::api_url!(
            "moderation/moderators?broadcaster_id={user_id}&first=100"
        ))
        .await?;

        let subs = TwitchVips::from_api(crate::api_url!(
            "subscriptions?broadcaster_id={user_id}&first=100"
        ))
        .await?;

        let users = TwitchUsers::new().await?;

        let users = users
            .data
            .par_iter()
            .map(|user| {
                let mut pooled_user: TwitchUser = TwitchUser {
                    name: user.from_name.clone(),
                    uid: user.from_id.clone(),
                    color: Color::generate_light(),
                    is_mod: false,
                    is_vip: false,
                    is_sub: false,
                };

                if vips.data.par_iter().any(|vip| vip.user_id == user.from_id) {
                    pooled_user.is_vip = true;
                }

                if mods
                    .data
                    .par_iter()
                    .any(|moderator| moderator.user_id == user.from_id)
                {
                    pooled_user.is_mod = true;
                }

                if subs.data.par_iter().any(|sub| sub.user_id == user.from_id) {
                    pooled_user.is_sub = true;
                }

                pooled_user
            })
            .collect::<Vec<TwitchUser>>();

        Ok(UserPool { users })
    }

    #[allow(clippy::missing_panics_doc)]
    pub fn send_message(&self, message: impl AsRef<str>) -> String {
        let mut rng = rand::thread_rng();
        let user = self.users.choose(&mut rng).unwrap();

        user.send_message(message)
    }

    pub fn send_message_as(&self, message: impl AsRef<str>, user: &TwitchUser) -> String {
        user.send_message(message)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TwitchUsers {
    pub total: usize,
    pub data: Vec<Datum>,
    pub pagination: Pagination,
}

impl TwitchUsers {
    pub async fn new() -> anyhow::Result<Self> {
        let mut total = 0;
        let mut pag_cursor = None;
        let mut data = vec![];

        loop {
            let mut url = crate::api_url!("users/follows?to_id={user_id}&first=100").to_string();

            if let Some(cursor) = pag_cursor {
                url.push_str(&format!("&after={cursor}"));
            }

            let result: TwitchUsers = CLIENT.get(url).send().await?.json().await?;

            // Not good to set it every single time but it's fine for now
            total += result.data.len();
            data.extend(result.data);

            if let Some(cursor) = result.pagination.cursor {
                pag_cursor = Some(cursor);
            } else {
                break;
            }
        }

        Ok(Self {
            total,
            data,
            pagination: Pagination::default(),
        })
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Datum {
    pub from_id: String,
    pub from_login: String,
    pub from_name: String,
    pub to_id: String,
    pub to_login: String,
    pub to_name: String,
    pub followed_at: String,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Pagination {
    pub cursor: Option<String>,
}
