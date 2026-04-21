use zenoh_pico_sys::z_string_empty;

use crate::zvalue::zvalue;

#[zvalue(name = "string", zdefault(zfn = z_string_empty))]
pub struct ZString;
