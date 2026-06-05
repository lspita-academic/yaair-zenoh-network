use thiserror::Error;
use zenoh_pico_sys::{_z_res_t_Z_OK, z_result_t};

// https://github.com/eclipse-zenoh/zenoh-pico/blob/3b3ab65cadbb10a8d7f32ba04cb15c26b8435dd5/include/zenoh-pico/utils/result.h#L37
#[derive(Debug, Error)]
pub enum ZenohError {
    #[error("Zenoh operation failed with code: {0}")]
    Generic(i8),
}

impl Default for ZenohError {
    fn default() -> Self {
        Self::Generic(-1)
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
            Err(ZenohError::Generic(self))
        }
    }
}
