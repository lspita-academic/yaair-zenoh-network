use std::{fmt::Display, time::Duration};

use static_cell::StaticCell;
use uuid::Uuid;
use zenoh_pico::{
    config::{ConfigBuilder, ConfigMode},
    session::Session,
};

static ZENOH_SESSION: StaticCell<Session> = StaticCell::new();

pub fn start_session<IfName: Display>(
    if_name: IfName,
    session_zid: Option<Uuid>,
) -> &'static mut Session {
    let mut config_builder = ConfigBuilder::default()
        .mode(ConfigMode::Peer)
        .scouting_timeout(Duration::from_secs(30))
        .multicast_locator(&format!("udp/224.0.0.224:7446#iface={if_name}"))
        .listen(&format!("udp/224.0.0.224:7447#iface={if_name}"));
    if let Some(zid) = session_zid {
        config_builder = config_builder.session_zid(zid);
    }
    let zenoh_config = config_builder
        .build()
        .expect("Failed to build Zenoh config");

    let zenoh_session = ZENOH_SESSION
        .init(Session::open(zenoh_config, None).expect("Failed to open zenoh session"));

    log::info!("Zenoh session id: {}", zenoh_session.zid());
    zenoh_session
}
