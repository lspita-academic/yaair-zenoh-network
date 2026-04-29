use std::fmt::Display;

use zenoh_pico_core::{result::IntoZenohResult, sys::z_id_to_string, zvalue::ZValue};
use zenoh_pico_macros::zwrap;

use crate::zstring::ZString;

#[zwrap(base = "id")]
pub struct ZId;

impl Display for ZId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let string = ZString::initialize(|zs| unsafe {
            z_id_to_string(&self.0, zs)
                .into_zresult()
                .expect("System out of memory");
        });
        string.fmt(f)
    }
}
