use std::{borrow::Cow, fmt::Display, net::SocketAddrV4, time::Duration};

use net_literals::addrv4;
use strum::Display;

use crate::ZenohNodeId;
pub use crate::zenoh::config::{ZenohConfig, ZenohConfigBuilder, ZenohConfigBuilderOptions};

pub type ConfigString = Cow<'static, str>;

#[derive(Display, Default)]
#[strum(serialize_all = "lowercase")]
pub enum LocatorProtocol {
    TCP,
    #[default]
    UDP,
}

pub struct Locator {
    pub protocol: LocatorProtocol,
    pub address: SocketAddrV4,
    pub interface: Option<ConfigString>,
}

impl From<SocketAddrV4> for Locator {
    fn from(address: SocketAddrV4) -> Self {
        Self {
            address,
            protocol: Default::default(),
            interface: Default::default(),
        }
    }
}

impl Display for Locator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}/{}", self.protocol, self.address)?;
        if let Some(ref interface) = self.interface {
            write!(f, "#iface={interface}")?;
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Display, PartialEq, Eq, Hash)]
#[strum(serialize_all = "lowercase")]
pub enum PeerType {
    Peer,
    Client,
    Router,
}

pub trait ConfigBuilder: Sized {
    type Err;
    type Config;
    type InitOptions;

    fn uninitialized(options: Self::InitOptions) -> Self;
    fn id(self, id: ZenohNodeId) -> Self;
    fn peer_type(self, peer_type: PeerType) -> Self;
    fn connect<L: Into<Locator>>(self, locator: L) -> Self;
    fn listen<L: Into<Locator>>(self, locator: L) -> Self;
    fn multicast_scouting(self, enable: bool) -> Self;
    fn multicast_locator<L: Into<Locator>>(self, locator: L) -> Self;
    fn scouting_timeout(self, timeout: Duration) -> Self;
    fn scouting_mask<PeerTypes: AsRef<[PeerType]>>(self, peer_types: PeerTypes) -> Self;
    fn build(self) -> Result<Self::Config, Self::Err>;

    fn new(options: Self::InitOptions) -> Self {
        Self::uninitialized(options)
            .peer_type(PeerType::Peer)
            .multicast_scouting(true)
            .scouting_timeout(Duration::from_secs(30))
            .multicast_locator(addrv4!("224.0.0.224:7446"))
            .listen(addrv4!("224.0.0.224:7447"))
    }
}

pub trait ConfigBuilderDefault {
    fn default() -> Self;
}

impl<T: ConfigBuilder> ConfigBuilderDefault for T
where
    T::InitOptions: Default,
{
    fn default() -> Self {
        Self::new(Default::default())
    }
}

pub struct NetworkConfig {
    pub base_keyexpr: ConfigString,
    pub lifespan: Duration,
    pub zenoh: ZenohConfig,
}

impl From<ZenohConfig> for NetworkConfig {
    fn from(zenoh: ZenohConfig) -> Self {
        Self {
            zenoh,
            base_keyexpr: "yaair/network/zenoh".into(),
            lifespan: Duration::from_secs(10),
        }
    }
}
