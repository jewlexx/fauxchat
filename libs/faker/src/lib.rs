#![warn(clippy::all, clippy::pedantic)]
#![allow(clippy::unsafe_derive_deserialize, clippy::missing_errors_doc)]

use std::net::{Ipv4Addr, SocketAddrV4};

#[must_use]
pub const fn url() -> Ipv4Addr {
    Ipv4Addr::new(127, 0, 0, 1)
}

#[must_use]
pub fn port() -> u16 {
    let port_var = std::env::var("FAUXCHAT_PORT").unwrap_or("8080".to_string());
    port_var.parse().expect("valid port string")
}

#[must_use]
pub fn addr() -> std::net::SocketAddr {
    std::net::SocketAddr::V4(SocketAddrV4::new(url(), port()))
}
