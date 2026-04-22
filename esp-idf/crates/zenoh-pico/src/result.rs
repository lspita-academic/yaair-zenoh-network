use crate::sys::{_z_res_t_Z_OK, z_result_t};

pub type ZenohError = i8;

pub trait ZResult<T> {
    fn zresult(self, value: T) -> Result<T, ZenohError>;
}

impl<T> ZResult<T> for z_result_t {
    fn zresult(self, value: T) -> Result<T, ZenohError> {
        if self as i32 == _z_res_t_Z_OK {
            Ok(value)
        } else {
            Err(self)
        }
    }
}
