use std::fmt::Display;

use zenoh_pico_core::{
    result::IntoZenohResult,
    sys::z_id_to_string,
    zvalue::{ZOwn, ZValue},
};
use zenoh_pico_macros::zwrap;

use crate::zstring::ZString;

#[zwrap(base(name = "id"), zvalue)]
pub struct ZId;

impl Display for ZId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut string = ZString::uninitialized();
        string
            .inspect_zowned_mut(|z| unsafe { z_id_to_string(&self.0, z).into_zresult() })
            .expect("System out of memory");
        string.fmt(f)
    }
}
