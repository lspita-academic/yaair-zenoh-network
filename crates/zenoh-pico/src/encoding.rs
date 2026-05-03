use std::{
    fmt::{self, Display},
    str::FromStr,
};

use zenoh_pico_macros::zwrap;
use zenoh_pico_sys::{
    ZP_ENCODING_ZENOH_BYTES, z_encoding_equals, z_encoding_from_substr, z_encoding_to_string,
};

use crate::{
    result::{IntoZenohResult, ZenohError},
    zstring::ZString,
    zvalue::{ZOwn, ZValue},
};

#[zwrap(base(name = "encoding"), zvalue, zown)]
pub struct Encoding;

impl Default for Encoding {
    fn default() -> Self {
        Self::from_zowned(unsafe { ZP_ENCODING_ZENOH_BYTES })
    }
}

impl FromStr for Encoding {
    type Err = ZenohError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut encoding = Self::uninitialized();
        encoding
            .with_zowned_mut(|z| unsafe {
                z_encoding_from_substr(z, s.as_ptr(), s.len()).into_zresult()
            })
            .map(|_| encoding)
    }
}

impl Display for Encoding {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut string = ZString::uninitialized();
        string
            .with_zowned_mut(|z| unsafe { z_encoding_to_string(self.zloan(), z).into_zresult() })
            .map_err(|_| fmt::Error)
            .and_then(|_| string.fmt(f))
    }
}

impl Eq for Encoding {}
impl PartialEq for Encoding {
    fn eq(&self, other: &Self) -> bool {
        unsafe { z_encoding_equals(self.zloan(), other.zloan()) }
    }
}
