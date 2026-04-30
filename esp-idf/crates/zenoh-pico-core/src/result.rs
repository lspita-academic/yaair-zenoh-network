use std::{error::Error, fmt::Display};

use zenoh_pico_sys::{_z_res_t_Z_OK, z_result_t};

#[derive(Debug)]
pub struct ZenohError(i8);

impl Error for ZenohError {}
impl Display for ZenohError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "zenoh operation failed with code: {}", self.0)
    }
}

pub type ZenohResult<T> = Result<T, ZenohError>;

pub trait IntoZenohResult<T> {
    fn into_zresult(self) -> ZenohResult<T>;
}

impl IntoZenohResult<()> for z_result_t {
    fn into_zresult(self) -> ZenohResult<()> {
        if self as i32 == _z_res_t_Z_OK {
            Ok(())
        } else {
            Err(ZenohError(self))
        }
    }
}
