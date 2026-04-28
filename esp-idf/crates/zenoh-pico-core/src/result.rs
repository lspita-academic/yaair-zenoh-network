use zenoh_pico_sys::{_z_res_t_Z_OK, z_result_t};

pub type ZenohError = i8;
pub type ZenohResult<T> = Result<T, ZenohError>;

pub trait IntoZenohResult<T> {
    fn into_zresult(self) -> ZenohResult<T>;
}

impl IntoZenohResult<()> for z_result_t {
    fn into_zresult(self) -> ZenohResult<()> {
        if self as i32 == _z_res_t_Z_OK {
            Ok(())
        } else {
            Err(self)
        }
    }
}
