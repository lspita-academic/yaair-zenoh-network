use zenoh_pico_core::sys::{z_string_empty, z_view_string_empty};

use crate::{zown, zview};

#[zown(base = "string", zdefault(zfn = z_string_empty), zloan(mutable))]
pub struct ZString;


#[zview(base = "string", zdefault(zfn = z_view_string_empty), zloan(mutable))]
pub struct ZViewString;
