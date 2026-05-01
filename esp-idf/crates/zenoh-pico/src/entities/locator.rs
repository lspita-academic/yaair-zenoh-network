use std::{fmt::Display, str::FromStr};

use zenoh_pico_core::{
    result::{IntoZenohResult, ZenohError},
    sys::{_z_locator_clear, _z_locator_from_string, _z_locator_to_string},
    zvalue::ZValue,
};
use zenoh_pico_macros::zwrap;

use crate::zstring::{FromZStr, ZString};

/// The locator struct acts like an owned value but it's only for internal use,
/// so the ownership function do not exist.
/// Zenoh pico expect the locator to be provided as a string, this wrapper is created
/// to add parse logic with a result.
#[zwrap(base(name = "locator"), zvalue)]
pub struct Locator;

impl Drop for Locator {
    fn drop(&mut self) {
        unsafe {
            _z_locator_clear(self.zloan_mut());
        }
    }
}

impl FromZStr for Locator {
    type Error = ZenohError;

    fn from_zstr(s: &ZString) -> Result<Self, Self::Error> {
        let mut value = Self::uninitialized();
        unsafe { _z_locator_from_string(value.zloan_mut(), s.zloan()).into_zresult() }
            .map(|_| value)
    }
}

impl FromStr for Locator {
    type Err = <Self as FromZStr>::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s: ZString = s.parse()?;
        Self::from_zstr(&s)
    }
}

impl Display for Locator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        ZString::from_zvalue(unsafe { _z_locator_to_string(self.zloan()) }).fmt(f)
    }
}
