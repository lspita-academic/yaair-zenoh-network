use std::{collections::HashMap, time::Duration};

use itertools::Itertools;
use serde_json::json;
pub use zenoh::config::Config as ZenohConfig;
use zenoh::config::WhatAmI;

use crate::{
    ZenohNodeId,
    config::{ConfigBuilder, Locator, PeerType},
    zenoh_impl::ZenohError,
};

impl From<PeerType> for WhatAmI {
    fn from(value: PeerType) -> Self {
        match value {
            PeerType::Peer => Self::Peer,
            PeerType::Client => Self::Client,
            PeerType::Router => Self::Router,
        }
    }
}

// <https://github.com/eclipse-zenoh/zenoh-pico/tree/1.9.0#34-basic-pubsub-example---p2p-over-udp-multicast>
const ZENOH_PICO_BATCH_MULTICAST_SIZE: usize = 2048;

pub struct ZenohConfigBuilderInitOptions {
    pub batch_multicast_size: usize,
}

impl Default for ZenohConfigBuilderInitOptions {
    fn default() -> Self {
        Self {
            batch_multicast_size: ZENOH_PICO_BATCH_MULTICAST_SIZE,
        }
    }
}

pub struct ZenohConfigBuilder {
    options_map: HashMap<String, serde_json::Value>,
}

impl ZenohConfigBuilder {
    fn option_insert<K: ToString>(mut self, key: K, value: serde_json::Value) -> Self {
        self.options_map.insert(key.to_string(), value);
        self
    }
}

impl ConfigBuilder for ZenohConfigBuilder {
    type Err = ZenohError;
    type Config = ZenohConfig;
    type InitOptions = ZenohConfigBuilderInitOptions;

    fn new(options: Self::InitOptions) -> Self {
        let builder = Self {
            options_map: Default::default(),
        };
        builder.option_insert(
            "transport/link/tx/batch_size",
            json!(options.batch_multicast_size),
        )
    }

    fn id(self, id: ZenohNodeId) -> Self {
        self.option_insert("id", json!(id.to_string()))
    }

    fn peer_type(self, peer_type: PeerType) -> Self {
        self.option_insert("mode", json!(peer_type.to_string()))
    }

    fn connect<L: Into<Locator>>(self, locator: L) -> Self {
        self.option_insert("connect/endpoints", json!([locator.into().to_string()]))
    }

    fn listen<L: Into<Locator>>(self, locator: L) -> Self {
        self.option_insert("listen/endpoints", json!([locator.into().to_string()]))
    }

    fn multicast_scouting(self, enable: bool) -> Self {
        self.option_insert("scouting/multicast/enabled", json!(enable))
    }

    fn multicast_locator<L: Into<Locator>>(mut self, locator: L) -> Self {
        let Locator {
            address, interface, ..
        } = locator.into();
        self = self.option_insert("scouting/multicast/address", json!(address.to_string()));
        if let Some(interface) = interface {
            self = self.option_insert("scouting/multicast/interface", json!(interface.to_string()));
        }
        self
    }

    fn scouting_timeout(self, timeout: Duration) -> Self {
        self.option_insert("scouting/timeout", json!(timeout.as_millis()))
    }

    fn scouting_mask<PeerTypes: AsRef<[PeerType]>>(self, peer_types: PeerTypes) -> Self {
        let what_mask: Vec<_> = peer_types
            .as_ref()
            .iter()
            .cloned()
            .unique()
            .map(|p| p.to_string())
            .collect();
        self.option_insert("scouting/multicast/autoconnect", json!(what_mask))
    }

    fn build(self) -> Result<Self::Config, Self::Err> {
        let mut config = ZenohConfig::default();
        self.options_map
            .into_iter()
            .try_for_each(|(key, value)| config.insert_json5(&key, &value.to_string()))
            .map(|_| config)
    }
}
