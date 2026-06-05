use std::{borrow::Cow, net::SocketAddrV4, time::Duration};

use strum::Display;

#[cfg(zenoh_impl = "zenoh_full")]
#[path = "zenoh_full.rs"]
pub mod zenoh;

#[cfg(zenoh_impl = "zenoh_pico")]
#[path = "zenoh_pico.rs"]
pub mod zenoh;

pub use zenoh::{ZenohConfig, ZenohConfigBuilder};

#[derive(Display)]
#[strum(serialize_all = "lowercase")]
pub enum LocatorProtocol {
    TCP,
    UDP,
}

pub struct Locator {
    protocol: LocatorProtocol,
    endpoint: SocketAddrV4,
    interface: Option<Cow<'static, str>>,
}

pub enum PeerType {
    Peer,
    Client,
    Router,
}

pub trait ConfigBuilder: Sized {
    type Err;
    type Config;

    fn new() -> Self;
    fn peer_type(self, peer_type: PeerType) -> Self;
    fn connect(self, locator: Locator) -> Self;
    fn listen(self, locator: Locator) -> Self;
    fn multicast_scouting(self, enable: bool) -> Self;
    fn multicast_locator(self, locator: Locator) -> Self;
    fn scouting_timeout(self, timeout: Duration) -> Self;
    fn scouting_include(self, peer_type: PeerType) -> Self;
    fn scouting_exclude(self, peer_type: PeerType) -> Self;
    fn build(self) -> Result<Self::Config, Self::Err>;
}

pub struct NetworkConfig {
    pub base_keyexpr: Cow<'static, str>,
    pub lifespan: Duration,
    pub zenoh: Config,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            base_keyexpr: "yaair/network/zenoh".into(),
            lifespan: Duration::from_secs(10),
            zenoh: Default::default(),
        }
    }
}
