use std::{marker::PhantomData, mem::MaybeUninit};

use zenoh_pico_macros::zwrap;
use zenoh_pico_sys::{
    z_bytes_copy_from_buf, z_bytes_empty, z_bytes_get_reader, z_bytes_reader_read, z_bytes_reader_t,
};

use crate::{
    result::{IntoZenohResult, ZenohError},
    zvalue::{ZOwn, ZValue},
};

#[zwrap(base(name = "bytes"), zvalue, zown)]
pub struct ZBytes;

impl Default for ZBytes {
    fn default() -> Self {
        let mut zbytes = Self::uninitialized();
        zbytes.with_zowned_mut(|z| unsafe { z_bytes_empty(z) });
        zbytes
    }
}

pub trait IntoZBytes {
    fn into_zbytes(self) -> ZBytes;
}

pub trait TryIntoZBytes {
    type Err;

    fn try_into_zbytes(self) -> Result<ZBytes, Self::Err>;
}

pub trait FromZBytes
where
    Self: Sized,
{
    type Err;

    fn from_zbytes(bytes: &ZBytes) -> Result<Self, Self::Err>;
}

impl IntoZBytes for ZBytes {
    fn into_zbytes(self) -> ZBytes {
        self
    }
}

impl ZBytes {
    pub fn parse<T: FromZBytes>(&self) -> Result<T, T::Err> {
        T::from_zbytes(&self)
    }
}

impl<T: AsRef<[u8]>> TryIntoZBytes for T {
    type Err = ZenohError;

    fn try_into_zbytes(self) -> Result<ZBytes, Self::Err> {
        let slice = self.as_ref();
        let mut zbytes = ZBytes::uninitialized();
        zbytes
            .with_zowned_mut(|z| unsafe {
                z_bytes_copy_from_buf(z, slice.as_ptr(), slice.len()).into_zresult()
            })?;
        Ok(zbytes)
    }
}

pub struct ZBytesIter<'a> {
    // zbytes: &'a ZBytes,
    reader: z_bytes_reader_t,
    phantom: PhantomData<&'a ZBytes>,
}

impl<'a> From<&'a ZBytes> for ZBytesIter<'a> {
    fn from(zbytes: &'a ZBytes) -> Self {
        let reader = unsafe { z_bytes_get_reader(zbytes.zloan()) };
        Self {
            reader,
            phantom: PhantomData,
        }
    }
}

impl<'a> Iterator for ZBytesIter<'a> {
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        let mut b = MaybeUninit::uninit();
        let bytes_read = unsafe { z_bytes_reader_read(&mut self.reader, b.as_mut_ptr(), 1) };
        if bytes_read > 0 {
            Some(unsafe { b.assume_init() })
        } else {
            None
        }
    }
}

impl<'a> IntoIterator for &'a ZBytes {
    type Item = <Self::IntoIter as Iterator>::Item;
    type IntoIter = ZBytesIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        Self::IntoIter::from(self)
    }
}
