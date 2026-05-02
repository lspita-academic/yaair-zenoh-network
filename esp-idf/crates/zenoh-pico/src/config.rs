use std::{ffi::CString, time::Duration};

use ffi_utils::cstring::CStringExtensions;
use num_enum::{IntoPrimitive, TryFromPrimitive, UnsafeFromPrimitive};
use strum::{Display, EnumString};
use zenoh_pico_core::{
    result::{IntoZenohResult, ZenohResult},
    sys::{
        Z_CONFIG_CONNECT_KEY, Z_CONFIG_LISTEN_KEY, Z_CONFIG_MODE_KEY,
        Z_CONFIG_MULTICAST_LOCATOR_KEY, Z_CONFIG_MULTICAST_SCOUTING_KEY,
        Z_CONFIG_SCOUTING_TIMEOUT_KEY, Z_CONFIG_SCOUTING_WHAT_KEY, z_config_default,
        zp_config_insert,
    },
    zvalue::{ZOwn, ZValue},
};
use zenoh_pico_macros::zwrap;

use crate::entities::{locator::Locator, whatami::WhatAmIMask};

#[derive(Debug, Default, EnumString, Display)]
#[strum(serialize_all = "snake_case")]
pub enum ZenohConfigMode {
    #[default]
    Client,
    Peer,
}

#[derive(IntoPrimitive, TryFromPrimitive, UnsafeFromPrimitive)]
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

#[zwrap(base(name = "config"), zvalue, zown, zclone)]
pub struct ZenohConfig;

pub struct ZenohConfigBuilder(ZenohConfig);

impl ZenohConfigBuilder {
    pub fn new() -> ZenohResult<Self> {
        let mut config = ZenohConfig::uninitialized();
        config
            .inspect_zowned_mut(|z| unsafe { z_config_default(z).into_zresult() })
            .map(|_| Self(config))
    }

    fn set<V: AsRef<str>>(mut self, key: ZenohConfigKey, value: V) -> ZenohResult<Self> {
        let value_cstring = CString::from_vec_maybe_nul(value.as_ref());
        unsafe {
            zp_config_insert(self.0.zloan_mut(), key.into(), value_cstring.as_ptr())
                .into_zresult()?;
        }
        Ok(self)
    }

    pub fn mode(self, mode: &ZenohConfigMode) -> ZenohResult<Self> {
        self.set(ZenohConfigKey::ConfigMode, mode.to_string())
    }

    pub fn connect(self, locator: &Locator) -> ZenohResult<Self> {
        self.set(ZenohConfigKey::Connect, locator.to_string())
    }

    pub fn listen(self, locator: &Locator) -> ZenohResult<Self> {
        self.set(ZenohConfigKey::Listen, locator.to_string())
    }

    pub fn scouting_timeout(self, timeout: &Duration) -> ZenohResult<Self> {
        self.set(
            ZenohConfigKey::ScoutingTimeout,
            timeout.as_millis().to_string(),
        )
    }

    pub fn multicast_scouting(self, enable: bool) -> ZenohResult<Self> {
        self.set(ZenohConfigKey::MulticastScouting, enable.to_string())
    }

    pub fn multicast_locator(self, locator: &Locator) -> ZenohResult<Self> {
        self.set(ZenohConfigKey::MulticastLocator, locator.to_string())
    }

    pub fn scouting_mask(self, what_mask: &WhatAmIMask) -> ZenohResult<Self> {
        self.set(ZenohConfigKey::ScoutingMask, what_mask.to_string())
    }

    pub fn build(self) -> ZenohConfig {
        self.0
    }
}
