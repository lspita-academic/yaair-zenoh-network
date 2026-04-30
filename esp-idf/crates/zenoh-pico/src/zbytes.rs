use std::{iter, mem::MaybeUninit};

use zenoh_pico_core::{
    result::ZenohError,
    sys::{z_bytes_copy_from_buf, z_bytes_empty, z_bytes_get_reader, z_bytes_reader_read},
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

impl FromZBytes for Vec<u8> {
    type Error = ZenohError;

    fn from_zbytes(bytes: &ZBytes) -> Result<Self, Self::Error> {
        let mut reader = unsafe { z_bytes_get_reader(bytes.zloan()) };
        Ok(iter::from_fn(move || {
            let mut b = MaybeUninit::uninit();
            let bytes_read = unsafe { z_bytes_reader_read(&mut reader, b.as_mut_ptr(), 1) };
            if bytes_read > 0 {
                Some(unsafe { b.assume_init() })
            } else {
                None
            }
        })
        .collect())
    }
}
