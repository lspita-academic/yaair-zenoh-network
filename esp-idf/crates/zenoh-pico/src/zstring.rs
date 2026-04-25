use zenoh_pico_core::sys::z_string_empty;

use crate::zown;

#[zown(name = "string", zdefault(zfn = z_string_empty), zloan(mutable))]
pub struct ZString;
