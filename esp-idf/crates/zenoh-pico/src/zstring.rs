use std::{
    ffi::CStr,
    fmt::{Display, Pointer},
    str::FromStr,
};

use zenoh_pico_core::{
    result::{IntoZenohResult, ZenohError},
    sys::{z_string_copy_from_substr, z_string_data},
    zvalue::{ZOwn, ZValue},
};
use zenoh_pico_macros::zwrap;

#[zwrap(base(name = "string"), zvalue, zown)]
pub struct ZString;

impl FromStr for ZString {
    type Err = ZenohError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut zstring = Self::uninitialized();
        zstring
            .inspect_zowned_mut(|z| unsafe {
                z_string_copy_from_substr(z, s.as_ptr(), s.len()).into_zresult()
            })
            .map(|_| zstring)
    }
}

impl Display for ZString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let ptr = unsafe { z_string_data(self.zloan()) };
        unsafe { CStr::from_ptr(ptr) }.fmt(f)
    }
}
