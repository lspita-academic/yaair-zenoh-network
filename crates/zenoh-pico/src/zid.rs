use std::fmt::{self, Display};

use zenoh_pico_macros::zwrap;

use crate::{
    result::IntoZenohResult,
    sys::z_id_to_string,
    zstring::ZString,
    zvalue::{ZOwn, ZValue},
};

#[zwrap(base(name = "id"), zvalue)]
pub struct ZId;

impl Display for ZId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut string = ZString::uninitialized();
        string
            .with_zowned_mut(|z| unsafe { z_id_to_string(&self.0, z).into_zresult() })
            .map_err(|_| fmt::Error)
            .and_then(|_| string.fmt(f))
    }
}
