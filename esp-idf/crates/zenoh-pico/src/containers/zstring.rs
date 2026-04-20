use std::fmt::Debug;

use crate::zvalue::ZValue;
use zenoh_pico_sys::{
    z_loaned_string_t, z_moved_string_t, z_owned_string_t, z_string_drop, z_string_empty,
    z_string_loan, z_string_loan_mut, z_string_move,
};

#[derive(ZValue, Debug)]
#[zvalue(
    zdrop(zfn = z_string_drop),
    zmove(ty = z_moved_string_t, zfn = z_string_move),
    zloan(ty = z_loaned_string_t, zfn = z_string_loan, zfn_mut = z_string_loan_mut),
    zdefault(zfn = z_string_empty),
)]
pub struct ZString(z_owned_string_t);
