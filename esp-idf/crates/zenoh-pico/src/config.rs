use std::{borrow::Borrow, collections::BTreeMap, ffi::CString, time::Duration};

use ffi_utils::cstring::CStringExtensions;
use num_enum::{IntoPrimitive, TryFromPrimitive, UnsafeFromPrimitive};
use strum::{Display, EnumString};
use zenoh_pico_macros::zwrap;

use crate::{
    entities::{whatami::WhatAmIMask},
    result::{IntoZenohResult, ZenohResult},
    sys::{
        Z_CONFIG_CONNECT_KEY, Z_CONFIG_LISTEN_KEY, Z_CONFIG_MODE_KEY,
        Z_CONFIG_MULTICAST_LOCATOR_KEY, Z_CONFIG_MULTICAST_SCOUTING_KEY,
        Z_CONFIG_SCOUTING_TIMEOUT_KEY, Z_CONFIG_SCOUTING_WHAT_KEY, z_config_default,
        zp_config_insert,
    },
    zvalue::{ZOwn, ZValue},
};

#[derive(Debug, Default, EnumString, Display)]
#[strum(serialize_all = "snake_case")]
pub enum ZenohConfigMode {
    #[default]
    Client,
    Peer,
}

#[derive(IntoPrimitive, TryFromPrimitive, UnsafeFromPrimitive, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u32)]
pub enum ZenohConfigKey {
    ConfigMode = Z_CONFIG_MODE_KEY,
    Connect = Z_CONFIG_CONNECT_KEY,
    Listen = Z_CONFIG_LISTEN_KEY,
    ScoutingTimeout = Z_CONFIG_SCOUTING_TIMEOUT_KEY,
    MulticastScouting = Z_CONFIG_MULTICAST_SCOUTING_KEY,
    MulticastLocator = Z_CONFIG_MULTICAST_LOCATOR_KEY,
    ScoutingMask = Z_CONFIG_SCOUTING_WHAT_KEY,
}

impl Into<u8> for ZenohConfigKey {
    fn into(self) -> u8 {
        Into::<u32>::into(self) as u8
    }
}

#[zwrap(base(name = "config"), zvalue, zown)]
pub struct ZenohConfig;

#[derive(Default)]
pub struct ZenohConfigBuilder {
    options: BTreeMap<ZenohConfigKey, String>,
}

impl ZenohConfigBuilder {
    fn set(config: &mut ZenohConfig, key: ZenohConfigKey, value: String) -> ZenohResult<()> {
        let value_cstring = CString::from_vec_maybe_nul(value);
        unsafe {
            zp_config_insert(config.zloan_mut(), key.into(), value_cstring.as_ptr()).into_zresult()
        }
    }

    pub fn mode(mut self, mode: ZenohConfigMode) -> Self {
        self.options
            .insert(ZenohConfigKey::ConfigMode, mode.to_string());
        self
    }

    pub fn connect(mut self, locator: &str) -> Self {
        self.options
            .insert(ZenohConfigKey::Connect, locator.to_string());
        self
    }

    pub fn listen(mut self, locator: &str) -> Self {
        self.options
            .insert(ZenohConfigKey::Listen, locator.to_string());
        self
    }

    pub fn scouting_timeout(mut self, timeout: Duration) -> Self {
        self.options.insert(
            ZenohConfigKey::ScoutingTimeout,
            timeout.borrow().as_millis().to_string(),
        );
        self
    }

    pub fn multicast_scouting(mut self, enable: bool) -> Self {
        self.options
            .insert(ZenohConfigKey::MulticastScouting, enable.to_string());
        self
    }

    pub fn multicast_locator(mut self, locator: &str) -> Self {
        self.options
            .insert(ZenohConfigKey::MulticastLocator, locator.to_string());
        self
    }

    pub fn scouting_mask(mut self, what_mask: WhatAmIMask) -> Self {
        self.options
            .insert(ZenohConfigKey::ScoutingMask, what_mask.to_string());
        self
    }

    pub fn build(self) -> ZenohResult<ZenohConfig> {
        let mut config = ZenohConfig::uninitialized();
        config.with_zowned_mut(|z| unsafe { z_config_default(z).into_zresult() })?;
        self.options
            .into_iter()
            .try_for_each(|(key, value)| Self::set(&mut config, key, value))
            .map(|_| config)
    }
}
