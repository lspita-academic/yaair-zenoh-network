use std::{borrow::Borrow, ffi::CStr, time::Duration};

use strum::{Display, EnumString};
use zenoh_pico_core::{
    sys::{
        Z_CONFIG_CONNECT_KEY, Z_CONFIG_LISTEN_KEY, Z_CONFIG_MODE_KEY,
        Z_CONFIG_MULTICAST_LOCATOR_KEY, Z_CONFIG_MULTICAST_SCOUTING_KEY,
        Z_CONFIG_SCOUTING_TIMEOUT_KEY, Z_CONFIG_SCOUTING_WHAT_KEY, z_config_default,
    },
    zvalue::ZLoanMut,
};

use crate::{
    locator::Locator,
    result::{ZResult, ZenohError},
    sys::zp_config_insert,
    whatami::WhatAmIMask,
    zvalue,
};

#[derive(Debug, Default, EnumString, Display)]
#[strum(serialize_all = "snake_case")]
pub enum ZenohConfigMode {
    #[default]
    Client,
    Peer,
}

pub enum ZenohConfigKey {
    ConfigMode,
    Connect,
    Listen,
    ScoutingTimeout,
    MulticastScouting,
    MulticastLocator,
    ScoutingMask,
}

impl ZenohConfigKey {
    pub fn num_key(&self) -> u8 {
        (match self {
            Self::ConfigMode => Z_CONFIG_MODE_KEY,
            Self::Connect => Z_CONFIG_CONNECT_KEY,
            Self::Listen => Z_CONFIG_LISTEN_KEY,
            Self::ScoutingTimeout => Z_CONFIG_SCOUTING_TIMEOUT_KEY,
            Self::MulticastScouting => Z_CONFIG_MULTICAST_SCOUTING_KEY,
            Self::MulticastLocator => Z_CONFIG_MULTICAST_LOCATOR_KEY,
            Self::ScoutingMask => Z_CONFIG_SCOUTING_WHAT_KEY,
        }) as u8
    }
}

#[zvalue(name = "config", zdefault(zfn = z_config_default), zloan(mutable))]
pub struct ZenohConfig;

#[derive(Default)]
pub struct ZenohConfigBuilder(ZenohConfig);

impl ZenohConfigBuilder {
    fn set<V: Borrow<str>>(mut self, key: ZenohConfigKey, value: V) -> Result<Self, ZenohError> {
        let z_config = &mut self.0;
        let value_bytes = [value.borrow().as_bytes(), &[0]].concat();
        let value_cstr = CStr::from_bytes_until_nul(value_bytes.as_slice()).unwrap();
        unsafe {
            zp_config_insert(
                z_config.zloan_mut(),
                key.borrow().num_key(),
                value_cstr.as_ptr(),
            )
        }
        .zresult(self)
    }

    pub fn mode(self, mode: &ZenohConfigMode) -> Self {
        self.set(ZenohConfigKey::ConfigMode, mode.to_string())
            .unwrap()
    }

    pub fn connect(self, locator: &Locator) -> Self {
        self.set(ZenohConfigKey::Connect, locator.to_string())
            .unwrap()
    }

    pub fn listen(self, locator: &Locator) -> Self {
        self.set(ZenohConfigKey::Listen, locator.to_string())
            .unwrap()
    }

    pub fn scouting_timeout(self, timeout: &Duration) -> Self {
        self.set(
            ZenohConfigKey::ScoutingTimeout,
            timeout.as_millis().to_string(),
        )
        .unwrap()
    }

    pub fn multicast_scouting(self, enable: bool) -> Self {
        self.set(ZenohConfigKey::MulticastScouting, enable.to_string())
            .unwrap()
    }

    pub fn multicast_locator(self, locator: &Locator) -> Self {
        self.set(ZenohConfigKey::MulticastLocator, locator.to_string())
            .unwrap()
    }

    pub fn scouting_mask(self, what_mask: &WhatAmIMask) -> Self {
        self.set(ZenohConfigKey::ScoutingMask, what_mask.to_string())
            .unwrap()
    }

    pub fn build(self) -> ZenohConfig {
        self.0
    }
}
