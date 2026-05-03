use std::{
    fmt::{self, Display},
    str::FromStr,
};

use num_enum::{IntoPrimitive, TryFromPrimitive, UnsafeFromPrimitive};
use zenoh_pico_macros::zwrap;
use zenoh_pico_sys::{
    ZP_ENCODING_ZENOH_BYTES, z_congestion_control_t_Z_CONGESTION_CONTROL_BLOCK,
    z_congestion_control_t_Z_CONGESTION_CONTROL_DROP, z_encoding_equals, z_encoding_from_substr,
    z_encoding_to_string, z_priority_t__Z_PRIORITY_CONTROL, z_priority_t_Z_PRIORITY_BACKGROUND,
    z_priority_t_Z_PRIORITY_DATA, z_priority_t_Z_PRIORITY_DATA_HIGH,
    z_priority_t_Z_PRIORITY_DATA_LOW, z_priority_t_Z_PRIORITY_INTERACTIVE_HIGH,
    z_priority_t_Z_PRIORITY_INTERACTIVE_LOW, z_priority_t_Z_PRIORITY_REAL_TIME,
};

use crate::{
    result::{IntoZenohResult, ZenohError},
    zstring::ZString,
    zvalue::{ZOwn, ZValue},
};

#[derive(IntoPrimitive, TryFromPrimitive, UnsafeFromPrimitive, Default)]
#[repr(u32)]
pub enum MessagePriority {
    Control = z_priority_t__Z_PRIORITY_CONTROL,
    Realtime = z_priority_t_Z_PRIORITY_REAL_TIME,
    InteractiveHigh = z_priority_t_Z_PRIORITY_INTERACTIVE_HIGH,
    InteractiveLow = z_priority_t_Z_PRIORITY_INTERACTIVE_LOW,
    DataHigh = z_priority_t_Z_PRIORITY_DATA_HIGH,
    #[default]
    Data = z_priority_t_Z_PRIORITY_DATA,
    DataLow = z_priority_t_Z_PRIORITY_DATA_LOW,
    Background = z_priority_t_Z_PRIORITY_BACKGROUND,
}

#[derive(IntoPrimitive, TryFromPrimitive, UnsafeFromPrimitive, Default)]
#[repr(u32)]
pub enum CongestionControl {
    Block = z_congestion_control_t_Z_CONGESTION_CONTROL_BLOCK,
    #[default]
    Drop = z_congestion_control_t_Z_CONGESTION_CONTROL_DROP,
}

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
        encoding.with_zowned_mut(|z| unsafe {
            z_encoding_from_substr(z, s.as_ptr(), s.len()).into_zresult()
        })?;
        Ok(encoding)
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
