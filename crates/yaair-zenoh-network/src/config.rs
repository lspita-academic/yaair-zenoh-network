//! Cross-platform types and traits to configure both the
//! [`ZenohNetwork`](crate::ZenohNetwork) and the zenoh implementation used.

use std::{borrow::Cow, fmt::Display, net::SocketAddrV4, time::Duration};

use net_literals::addrv4;
use strum::Display;

use crate::ZenohNodeId;
/// A configuration for the zenoh implementation used by the
/// [`ZenohNetwork`](crate::ZenohNetwork).
///
/// - If Zenoh is used, than it corresponds to <zenoh::Config>.
/// - If Zenoh pico is used, than it corresponds to a wrapper around the C
///   config struct.
///
/// In both cases, this struct SHOULD be build from the [`ZenohConfigBuilder`]
/// and not further modified.
pub use crate::zenoh_impl::config::ZenohConfig;
/// A builder to create a [`ZenohConfig`].
pub use crate::zenoh_impl::config::ZenohConfigBuilder;
/// Initialization options to create a [`ZenohConfigBuilder`].
///
/// These represent the required options that must be provided upfront.
/// The shape of this struct can change based on the zenoh implementation being
/// used.
pub use crate::zenoh_impl::config::ZenohConfigBuilderInitOptions;

/// A string value used in config structs.
///
/// It allows to use both owned strings or static references.
pub type ConfigString = Cow<'static, str>;

/// An type of network protocol used by a [`Locator`].
#[derive(Display, Default)]
#[strum(serialize_all = "lowercase")]
pub enum LocatorProtocol {
    TCP,
    #[default]
    UDP,
}

/// An endpoint in the zenoh network.
pub struct Locator {
    /// The network protocol to use.
    pub protocol: LocatorProtocol,
    /// The socket address to connect to.
    pub address: SocketAddrV4,
    /// The network interface to use for the connection. If not specified, the
    /// default interface will be used.
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

/// The type of peer a node can be in a zenoh network.
///
/// **NOT ALL ZENOH IMPLEMENTATIONS SUPPORT ALL PEER TYPES.**
#[derive(Clone, Copy, Display, PartialEq, Eq, Hash)]
#[strum(serialize_all = "lowercase")]
pub enum PeerType {
    Peer,
    Client,
    Router,
}

/// A builder to create a zenoh configuration.
pub trait ConfigBuilder: Sized {
    /// The error type possibly returned on [`build`](Self::build).
    type Err;
    /// The configuration type produced by the builder.
    type Config;
    /// The options used to initialize the builder.
    type InitOptions;

    /// Initialize the builder with the given options.
    fn new(options: Self::InitOptions) -> Self;

    /// Set the node id.
    fn id(self, id: ZenohNodeId) -> Self;

    /// Set the node peer type.
    fn peer_type(self, peer_type: PeerType) -> Self;

    /// Connect to the given locator.
    fn connect<L: Into<Locator>>(self, locator: L) -> Self;

    /// Listen on the given locator.
    fn listen<L: Into<Locator>>(self, locator: L) -> Self;

    /// Enable or disable multicast scouting.
    fn multicast_scouting(self, enable: bool) -> Self;

    /// Set the multicast locator.
    fn multicast_locator<L: Into<Locator>>(self, locator: L) -> Self;

    /// Set the scouting timeout.
    fn scouting_timeout(self, timeout: Duration) -> Self;

    /// Set a filter on which [peer types](PeerType) to scout for.
    fn scouting_mask<PeerTypes: AsRef<[PeerType]>>(self, peer_types: PeerTypes) -> Self;

    /// Build the configuration.
    fn build(self) -> Result<Self::Config, Self::Err>;

    /// Set the default options for the configuration.
    fn set_default_options(self) -> Self {
        self.peer_type(PeerType::Peer)
            .multicast_scouting(true)
            .scouting_timeout(Duration::from_secs(30))
            .multicast_locator(addrv4!("224.0.0.224:7446"))
            .listen(addrv4!("224.0.0.224:7447"))
    }
}

/// Extension trait for [`ConfigBuilder`] types whose
/// [`InitOptions`](ConfigBuilder::InitOptions) implement
/// [`Default`](trait@Default).
///
/// This is a workaround for the orphan rule, which prevents a blanket
/// `impl<T: ConfigBuilder> Default for T`.
pub trait ConfigBuilderDefault {
    /// Creates a new builder with default initialization and builder options.
    fn with_default_options() -> Self;
}

impl<T: ConfigBuilder> ConfigBuilderDefault for T
where
    T::InitOptions: Default,
{
    fn with_default_options() -> Self {
        Self::new(Default::default()).set_default_options()
    }
}

/// A [`ZenohNetwork`](crate::ZenohNetwork) configuration options.
///
/// At least the zenoh config is needed, so no [`Default`](trait@Default)
/// implementation is provided.
///
/// ```compile_fail
/// let network_config = NetworkConfig {
///     lifespan: Duration::from_secs(10),
///     ..Default::default(), // this is not possible
/// };
/// ```
///
/// Instead, [`From<ZenohConfig>`](trait@From) is implemented for convenience.
///
/// ```no_run
/// let zenoh_config: ZenohConfig;
/// let network_config = NetworkConfig {
///     lifespan: Duration::from_secs(10),
///     ..zenoh_config.into(), // do this to complete with defaults
/// };
/// ```
#[derive(Clone)]
pub struct ZenohNetworkConfig {
    /// The base topic to use as a namespace to work under.
    pub base_keyexpr: ConfigString,

    /// The default lifespan of the nodes in the network.
    ///
    /// If no new messages or [heartbeats](crate::heartbeat) within this time from
    /// the last one, the node is considered offline.
    pub lifespan: Duration,

    /// The configuration to use for zenoh.
    pub zenoh: ZenohConfig,
}

impl From<ZenohConfig> for ZenohNetworkConfig {
    fn from(zenoh: ZenohConfig) -> Self {
        Self {
            zenoh,
            base_keyexpr: "yaair/network/zenoh".into(),
            lifespan: Duration::from_secs(10),
        }
    }
}
