use std::{borrow::Borrow, collections::BTreeMap, ffi::CString, time::Duration};

use ffi_utils::cstring::CStringExtensions;
use num_enum::{IntoPrimitive, TryFromPrimitive, UnsafeFromPrimitive};
use strum::{Display, EnumString};
use zenoh_pico_macros::zwrap;
use zenoh_pico_sys::{
    Z_CONFIG_CONNECT_KEY, Z_CONFIG_LISTEN_KEY, Z_CONFIG_MODE_KEY, Z_CONFIG_MULTICAST_LOCATOR_KEY,
    Z_CONFIG_MULTICAST_SCOUTING_KEY, Z_CONFIG_SCOUTING_TIMEOUT_KEY, Z_CONFIG_SCOUTING_WHAT_KEY,
    Z_CONFIG_SESSION_ZID_KEY, z_config_default, zp_config_insert,
};

use crate::{
    entities::whatami::WhatAmIMask,
    result::{IntoZenohResult, ZenohResult},
    zid::ZId,
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

#[zwrap(base(name = "config"), zvalue, zown, zclone)]
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

    pub fn session_zid(mut self, zid: ZId) -> Self {
        let bytes_string = hex::encode(zid.into_bytes());

        // The function that parses the config string into a uuid doesn't correctly
        // follow the uuidv4 specification, so manual conversion is needed
        //
        // RFC 4122:        XXXXXXXX-XXXX-XXXX-XXXX-XXXXXXXXXXXX (8-4-4-4-12)
        // zenoh pico uuid: XXXXXXXX-XXXX-XXXX-XX-XXXXXXXXXXXXXX (8-4-4-2-14)
        //
        // Permalink to the implementation:
        // <https://github.com/eclipse-zenoh/zenoh-pico/blob/07c84ebcf926114bffbf70edd82f3d71919c3868/src/utils/uuid.c#L26>
        //
        // Opened issue:
        // <https://github.com/eclipse-zenoh/zenoh-pico/issues/1229>
        let uuid_string = [
            &bytes_string[..8],    // 8
            &bytes_string[8..12],  // 4
            &bytes_string[12..16], // 4
            &bytes_string[16..18], // 2
            &bytes_string[18..],   // 14
        ]
        .join("-");
        self.options.insert(ConfigKey::SessionZId, uuid_string);
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
