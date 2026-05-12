use std::{borrow::Borrow, collections::BTreeMap, ffi::CString, time::Duration};

use ffi_utils::cstring::CStringExtensions;
use num_enum::{IntoPrimitive, TryFromPrimitive, UnsafeFromPrimitive};
use strum::{Display, EnumString};
use uuid::Uuid;
use zenoh_pico_macros::zwrap;
use zenoh_pico_sys::{
    Z_CONFIG_CONNECT_KEY, Z_CONFIG_LISTEN_KEY, Z_CONFIG_MODE_KEY, Z_CONFIG_MULTICAST_LOCATOR_KEY,
    Z_CONFIG_MULTICAST_SCOUTING_KEY, Z_CONFIG_SCOUTING_TIMEOUT_KEY, Z_CONFIG_SCOUTING_WHAT_KEY,
    Z_CONFIG_SESSION_ZID_KEY, z_config_default, zp_config_insert,
};

use crate::{
    entities::whatami::WhatAmIMask,
    result::{IntoZenohResult, ZenohResult},
    zvalue::{ZOwn, ZValue},
};

#[derive(Debug, Default, EnumString, Display)]
#[strum(serialize_all = "snake_case")]
pub enum ConfigMode {
    #[default]
    Client,
    Peer,
}

#[derive(IntoPrimitive, TryFromPrimitive, UnsafeFromPrimitive, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u32)]
pub enum ConfigKey {
    ConfigMode = Z_CONFIG_MODE_KEY,
    Connect = Z_CONFIG_CONNECT_KEY,
    Listen = Z_CONFIG_LISTEN_KEY,
    ScoutingTimeout = Z_CONFIG_SCOUTING_TIMEOUT_KEY,
    MulticastScouting = Z_CONFIG_MULTICAST_SCOUTING_KEY,
    MulticastLocator = Z_CONFIG_MULTICAST_LOCATOR_KEY,
    ScoutingMask = Z_CONFIG_SCOUTING_WHAT_KEY,
    SessionZId = Z_CONFIG_SESSION_ZID_KEY,
}

impl Into<u8> for ConfigKey {
    fn into(self) -> u8 {
        Into::<u32>::into(self) as u8
    }
}

#[zwrap(base(name = "config"), zvalue, zown)]
pub struct Config;

#[derive(Default)]
pub struct ConfigBuilder {
    options: BTreeMap<ConfigKey, String>,
}

impl ConfigBuilder {
    fn set(config: &mut Config, key: ConfigKey, value: String) -> ZenohResult<()> {
        let value_cstring = CString::from_vec_maybe_nul(value);
        unsafe {
            zp_config_insert(config.zloan_mut(), key.into(), value_cstring.as_ptr()).into_zresult()
        }
    }

    pub fn mode(mut self, mode: ConfigMode) -> Self {
        self.options.insert(ConfigKey::ConfigMode, mode.to_string());
        self
    }

    pub fn connect(mut self, locator: &str) -> Self {
        self.options.insert(ConfigKey::Connect, locator.to_string());
        self
    }

    pub fn listen(mut self, locator: &str) -> Self {
        self.options.insert(ConfigKey::Listen, locator.to_string());
        self
    }

    pub fn scouting_timeout(mut self, timeout: Duration) -> Self {
        self.options.insert(
            ConfigKey::ScoutingTimeout,
            timeout.borrow().as_millis().to_string(),
        );
        self
    }

    pub fn multicast_scouting(mut self, enable: bool) -> Self {
        self.options
            .insert(ConfigKey::MulticastScouting, enable.to_string());
        self
    }

    pub fn multicast_locator(mut self, locator: &str) -> Self {
        self.options
            .insert(ConfigKey::MulticastLocator, locator.to_string());
        self
    }

    pub fn scouting_mask(mut self, what_mask: WhatAmIMask) -> Self {
        self.options
            .insert(ConfigKey::ScoutingMask, what_mask.to_string());
        self
    }

    pub fn session_zid(mut self, zid: Uuid) -> Self {
        self.options.insert(ConfigKey::SessionZId, zid.to_string());
        self
    }

    pub fn build(self) -> ZenohResult<Config> {
        let mut config = Config::uninitialized();
        config.with_zowned_mut(|z| unsafe { z_config_default(z).into_zresult() })?;
        self.options
            .into_iter()
            .try_for_each(|(key, value)| Self::set(&mut config, key, value))?;
        Ok(config)
    }
}
