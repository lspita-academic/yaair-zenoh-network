use zenoh_pico_core::{
    sys::{z_bytes_copy_from_buf, z_bytes_empty},
    zvalue::{ZOwn, ZValue},
};
use zenoh_pico_macros::zwrap;

#[zwrap(base(name = "bytes"), zvalue, zown)]
pub struct ZBytes;

impl Default for ZBytes {
    fn default() -> Self {
        let mut value = Self::uninitialized();
        value.inspect_zowned_mut(|z| unsafe { z_bytes_empty(z) });
        value
    }
}

pub trait IntoZBytes {
    fn into_zbytes(self) -> ZBytes;
}

pub trait FromZBytes: Sized {
    type Error;

    fn from_zbytes(bytes: &ZBytes) -> Result<Self, Self::Error>;
}

impl<T: IntoZBytes> From<T> for ZBytes {
    fn from(value: T) -> Self {
        value.into_zbytes()
    }
}

impl ZBytes {
    pub fn parse<T: FromZBytes>(&self) -> Result<T, T::Error> {
        T::from_zbytes(&self)
    }
}

impl IntoZBytes for &[u8] {
    fn into_zbytes(self) -> ZBytes {
        let mut bytes = Default::default();
        unsafe {
            z_bytes_copy_from_buf(&mut bytes, self.as_ptr(), self.len());
        }
        ZBytes::from(bytes)
    }
}

impl IntoZBytes for &str {
    fn into_zbytes(self) -> ZBytes {
        ZBytes::from(self.as_bytes())
    }
}
