use std::{ffi::CString, str::FromStr};

use ffi_utils::cstring::CStringExtensions;
use zenoh_pico_core::sys::z_keyexpr_from_str;

use crate::{result::{ZResult, ZenohError}, zown};

#[zown(base = "keyexpr", zloan(mutable))]
pub struct KeyExpr;

impl FromStr for KeyExpr {
    type Err = ZenohError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let value = CString::from_vec_maybe_nul(s);
        let mut keyexpr = Default::default();
        unsafe {
            z_keyexpr_from_str(&mut keyexpr, value.as_ptr()).zresult(())?;
        }
        Ok(Self(keyexpr))
    }
}
