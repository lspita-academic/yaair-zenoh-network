use crate::zvalue::ZValue;
use zenoh_pico_sys::z_owned_string_t;

#[derive(ZValue)]
pub struct ZString(z_owned_string_t);
