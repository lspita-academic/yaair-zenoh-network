use std::time::Duration;

pub use zenoh_pico::config::Config as ZenohConfig;
use zenoh_pico::{
    config::{ConfigBuilder as ZenohPicoConfigBuilder, ConfigMode},
    entities::whatami::WhatAmI,
    result::ZenohError,
    zid::ZId,
};

use crate::{
    ZenohNodeId,
    comm::FromZenohNodeId,
    config::{ConfigBuilder, ConfigString, Locator, PeerType},
};

impl TryFrom<PeerType> for ConfigMode {
    type Error = ZenohError;

    fn try_from(value: PeerType) -> Result<Self, Self::Error> {
        match value {
            PeerType::Peer => Ok(Self::Peer),
            PeerType::Client => Ok(Self::Client),
            PeerType::Router => Err(Default::default()),
        }
    }
}

impl From<PeerType> for WhatAmI {
    fn from(value: PeerType) -> Self {
        match value {
            PeerType::Peer => Self::Peer,
            PeerType::Client => Self::Client,
            PeerType::Router => Self::Router,
        }
    }
}

pub struct ZenohConfigBuilderOptions {
    /// interface is required for multicast, and it's rare that multicast will
    /// be disabled, so it's not an [Option](Option) type.
    ///
    /// <https://github.com/eclipse-zenoh/zenoh-pico#34-basic-pubsub-example---p2p-over-udp-multicast>
    pub interface: ConfigString,
}

pub struct ZenohConfigBuilder {
    pico_builder: ZenohPicoConfigBuilder,
    peer_type: Option<PeerType>,
    options: ZenohConfigBuilderOptions,
}

impl ZenohConfigBuilder {
    fn locator_defaults<L: Into<Locator>>(&self, locator: L) -> Locator {
        let mut locator = locator.into();
        if locator.interface.is_none() {
            locator.interface = Some(self.options.interface.clone());
        }
        locator
    }
}

impl ConfigBuilder for ZenohConfigBuilder {
    type Err = ZenohError;
    type Config = ZenohConfig;
    type InitOptions = ZenohConfigBuilderOptions;

    fn uninitialized(options: Self::InitOptions) -> Self {
        Self {
            options,
            pico_builder: Default::default(),
            peer_type: Default::default(),
        }
    }

    fn id(mut self, id: ZenohNodeId) -> Self {
        self.pico_builder = self.pico_builder.session_zid(ZId::from_node_id(id));
        self
    }

    fn peer_type(mut self, peer_type: PeerType) -> Self {
        self.peer_type = Some(peer_type);
        self
    }

    fn connect<L: Into<Locator>>(mut self, locator: L) -> Self {
        let locator = self.locator_defaults(locator);
        self.pico_builder = self.pico_builder.connect(&locator.to_string());
        self
    }

    fn listen<L: Into<Locator>>(mut self, locator: L) -> Self {
        let locator = self.locator_defaults(locator);
        self.pico_builder = self.pico_builder.listen(&locator.to_string());
        self
    }

    fn multicast_scouting(mut self, enable: bool) -> Self {
        self.pico_builder = self.pico_builder.multicast_scouting(enable);
        self
    }

    fn multicast_locator<L: Into<Locator>>(mut self, locator: L) -> Self {
        let locator = self.locator_defaults(locator);
        self.pico_builder = self.pico_builder.multicast_locator(&locator.to_string());
        self
    }

    fn scouting_timeout(mut self, timeout: Duration) -> Self {
        self.pico_builder = self.pico_builder.scouting_timeout(timeout);
        self
    }

    fn scouting_mask<PeerTypes: AsRef<[PeerType]>>(mut self, peer_types: PeerTypes) -> Self {
        let what_mask = peer_types
            .as_ref()
            .iter()
            .cloned()
            .map(WhatAmI::from)
            .collect();
        self.pico_builder = self.pico_builder.scouting_mask(what_mask);
        self
    }

    fn build(self) -> Result<Self::Config, Self::Err> {
        let mut pico_builder = self.pico_builder;
        if let Some(peer_type) = self.peer_type {
            pico_builder = pico_builder.mode(peer_type.try_into()?);
        }
        pico_builder.build()
    }
}
