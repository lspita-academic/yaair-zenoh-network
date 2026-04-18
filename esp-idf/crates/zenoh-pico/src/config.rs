use std::{ffi::CStr, time::Duration};

use esp_idf_sys::zenoh_pico::{
    _z_res_t_Z_OK, Z_CONFIG_LISTEN_KEY, Z_CONFIG_MODE_KEY, Z_CONFIG_MULTICAST_LOCATOR_KEY,
    Z_CONFIG_SCOUTING_TIMEOUT_KEY, z_config_default, z_config_drop, z_config_loan,
    z_config_loan_mut, z_config_move, z_owned_config_t, zp_config_get, zp_config_insert,
};

#[derive(Debug)]
pub enum ZenohConfigMode {
    Peer,
}

impl Into<&'static str> for ZenohConfigMode {
    fn into(self) -> &'static str {
        match self {
            ZenohConfigMode::Peer => "peer",
        }
    }
}

#[derive(Debug)]
pub struct InvalidConfigModeError;

impl TryFrom<&str> for ZenohConfigMode {
    type Error = InvalidConfigModeError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "peer" => Ok(ZenohConfigMode::Peer),
            _ => Err(InvalidConfigModeError),
        }
    }
}

#[derive(Debug)]
pub enum ZenohConfigKey {
    Listen,
    MulticastLocator,
    Mode,
    ScoutingTimeout,
}

impl ZenohConfigKey {
    fn z_value(self) -> u8 {
        let key = match self {
            Self::Listen => Z_CONFIG_LISTEN_KEY,
            Self::MulticastLocator => Z_CONFIG_MULTICAST_LOCATOR_KEY,
            Self::Mode => Z_CONFIG_MODE_KEY,
            Self::ScoutingTimeout => Z_CONFIG_SCOUTING_TIMEOUT_KEY,
        };
        key as u8
    }
}

#[derive(Debug)]
pub struct ZenohConfig {
    pub(super) z_config: z_owned_config_t,
}

impl From<z_owned_config_t> for ZenohConfig {
    fn from(z_config: z_owned_config_t) -> Self {
        Self { z_config }
    }
}

impl Default for ZenohConfig {
    fn default() -> Self {
        let mut z_config = z_owned_config_t::default();
        let result = unsafe { z_config_default(&mut z_config) };
        assert!(
            result == 0,
            "Cannot create default zenoh config: {}",
            result
        );
        Self::from(z_config)
    }
}

impl Drop for ZenohConfig {
    fn drop(&mut self) {
        let z_config = &mut self.z_config;
        unsafe {
            z_config_drop(z_config_move(z_config));
        }
    }
}

impl ZenohConfig {
    fn get(&self, key: ZenohConfigKey) -> &str {
        let z_config = &self.z_config;
        let value_ptr = unsafe { zp_config_get(z_config_loan(z_config), key.z_value()) };
        assert!(!value_ptr.is_null(), "Invalid zenoh config key");
        let value_cstr = unsafe { CStr::from_ptr(value_ptr) };
        value_cstr.to_str().unwrap()
    }

    pub fn mode(&self) -> ZenohConfigMode {
        let value_str = self.get(ZenohConfigKey::Mode);
        ZenohConfigMode::try_from(value_str).unwrap()
    }
}

pub struct ZenohConfigBuilder {
    config: ZenohConfig,
}

impl Default for ZenohConfigBuilder {
    fn default() -> Self {
        Self {
            config: Default::default(),
        }
    }
}

impl ZenohConfigBuilder {
    fn set(mut self, key: ZenohConfigKey, value: &str) -> Result<Self, i8> {
        let z_config = &mut self.config.z_config;
        let value_bytes = [value.as_bytes(), &[0]].concat();
        let value_cstr = CStr::from_bytes_until_nul(value_bytes.as_slice()).unwrap();
        let result = unsafe {
            zp_config_insert(
                z_config_loan_mut(z_config),
                key.z_value(),
                value_cstr.as_ptr(),
            )
        };
        if result == _z_res_t_Z_OK as i8 {
            Ok(self)
        } else {
            Err(result)
        }
    }

    pub fn mode(self, mode: ZenohConfigMode) -> Self {
        self.set(ZenohConfigKey::Mode, mode.into()).unwrap()
    }

    pub fn scouting_timeout(self, timeout: Duration) -> Self {
        self.set(
            ZenohConfigKey::ScoutingTimeout,
            &timeout.as_millis().to_string(),
        )
        .unwrap()
    }

    pub fn listen(self, locator: &str) -> Self {
        self.set(ZenohConfigKey::Listen, locator).unwrap()
    }

    pub fn multicast_locator(self, locator: &str) -> Self {
        self.set(ZenohConfigKey::MulticastLocator, locator).unwrap()
    }

    pub fn build(self) -> ZenohConfig {
        self.config
    }
}
