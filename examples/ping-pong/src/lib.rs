#![cfg(target_os = "espidf")]

pub mod ping;
pub mod pong;

use std::{fmt::Display, time::Duration};

use static_cell::StaticCell;
use zenoh_pico::{
    config::{ConfigBuilder, ConfigMode},
    session::Session,
};

static ZENOH_SESSION: StaticCell<Session> = StaticCell::new();

pub fn start_zenoh_session<IfName: Display>(if_name: IfName) -> &'static mut Session {
    let zenoh_config = ConfigBuilder::default()
        .mode(ConfigMode::Peer)
        .scouting_timeout(Duration::from_secs(30))
        .multicast_locator(&format!("udp/224.0.0.224:7446#iface={if_name}"))
        .listen(&format!("udp/224.0.0.224:7447#iface={if_name}"))
        .build()
        .expect("Failed to build Zenoh config");

    let zenoh_session = ZENOH_SESSION
        .init(Session::open(zenoh_config, None).expect("Failed to open zenoh session"));

    log::info!("Zenoh session id: {}", zenoh_session.zid());
    zenoh_session
}
