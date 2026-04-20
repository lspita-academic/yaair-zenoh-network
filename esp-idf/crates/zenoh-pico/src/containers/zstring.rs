use crate::zvalue::ZValue;
use zenoh_pico_sys::{
    z_loaned_string_t, z_moved_string_t, z_owned_string_t, z_string_drop, z_string_empty,
    z_string_loan, z_string_loan_mut, z_string_move,
};

#[derive(ZValue)]
#[zdrop(z_string_drop)]
#[zmove(z_moved_string_t, z_string_move)]
#[zloan(z_loaned_string_t, z_string_loan, z_string_loan_mut)]
#[zdefault(z_string_empty)]
// #[zmove(z_moved_string_t, z_string_move)]
pub struct ZString(z_owned_string_t);
