use std::{ffi::CString, str::FromStr};

use ffi_utils::cstring::CStringExtensions;
use zenoh_pico_core::sys::{z_string_copy_from_str, z_string_empty, z_view_string_empty};

use crate::{
    result::{ZResult, ZenohError},
    zown, zview,
};

#[zown(base = "string", zdefault(zfn = z_string_empty), zloan(mutable))]
pub struct ZString;

impl FromStr for ZString {
    type Err = ZenohError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let value = CString::from_vec_maybe_nul(s);
        let mut zstring = Default::default();
        unsafe {
            z_string_copy_from_str(&mut zstring, value.as_ptr()).zresult(())?;
        }
        Ok(Self(zstring))
    }
}

#[zview(base = "string", zdefault(zfn = z_view_string_empty), zloan(mutable))]
pub struct ZStr;
