use std::{
    ffi::CStr,
    fmt::{Display, Pointer},
    str::FromStr,
};

use zenoh_pico_core::{
    result::{IntoZenohResult, ZenohError},
    sys::{z_string_copy_from_substr, z_string_data, z_string_empty},
    zvalue::ZLoan,
};
use zenoh_pico_macros::zown;

#[zown(base = "string", zdefault(zfn = z_string_empty), zloan(mutable))]
pub struct ZString;

impl FromStr for ZString {
    type Err = ZenohError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut zstring = Default::default();
        unsafe {
            z_string_copy_from_substr(&mut zstring, s.as_ptr(), s.len()).into_zresult()?;
        }
        Ok(Self::from(zstring))
    }
}

impl Display for ZString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let ptr = unsafe { z_string_data(self.zloan()) };
        unsafe { CStr::from_ptr(ptr) }.fmt(f)
    }
}
