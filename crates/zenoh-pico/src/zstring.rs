use std::{
    fmt::{self, Display},
    str::FromStr,
};

use zenoh_pico_macros::zwrap;
use zenoh_pico_sys::{
    _z_string_equals, _z_string_svec_t, z_string_array_get, z_string_array_len,
    z_string_copy_from_substr, z_string_data, z_string_len,
};

use crate::{
    result::{IntoZenohResult, ZenohError},
    zvalue::{ZOwn, ZValue},
};

#[zwrap(base(name = "string"), zvalue, zown, zclone)]
pub struct ZString;

pub trait FromZStr
where
    Self: Sized,
{
    type Err;

    fn from_zstr(s: &ZString) -> Result<Self, Self::Err>;
}

impl ZString {
    pub fn len(&self) -> usize {
        unsafe { z_string_len(self.zloan()) }
    }

    pub fn parse<T: FromZStr>(&self) -> Result<T, T::Err> {
        T::from_zstr(self)
    }
}

impl FromStr for ZString {
    type Err = ZenohError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut zstring = Self::uninitialized();
        zstring
            .with_zowned_mut(|z| unsafe {
                z_string_copy_from_substr(z, s.as_ptr(), s.len()).into_zresult()
            })
            .map(|_| zstring)
    }
}

impl Display for ZString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let slice = unsafe { std::slice::from_raw_parts(z_string_data(self.zloan()), self.len()) };
        str::from_utf8(&slice).map_err(|_| fmt::Error)?.fmt(f)
    }
}

impl Eq for ZString {}
impl PartialEq for ZString {
    fn eq(&self, other: &Self) -> bool {
        unsafe { _z_string_equals(self.zloan(), other.zloan()) }
    }
}

#[zwrap(base(name = "string_array"), zvalue(value_ty = _z_string_svec_t), zown)]
pub struct ZStringArray;

pub struct ZStringArrayIter<'a> {
    zstring_array: &'a ZStringArray,
    len: usize,
    index: usize,
}

impl<'a> From<&'a ZStringArray> for ZStringArrayIter<'a> {
    fn from(zstring_array: &'a ZStringArray) -> Self {
        let len = unsafe { z_string_array_len(zstring_array.zloan()) };
        Self {
            zstring_array,
            len,
            index: 0,
        }
    }
}

impl<'a> Iterator for ZStringArrayIter<'a> {
    type Item = &'a ZString;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.len {
            let zstring = ZString::from_ptr(unsafe {
                z_string_array_get(self.zstring_array.zloan(), self.index)
            });
            self.index += 1;
            Some(zstring)
        } else {
            None
        }
    }
}

impl<'a> IntoIterator for &'a ZStringArray {
    type Item = <Self::IntoIter as Iterator>::Item;
    type IntoIter = ZStringArrayIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        Self::IntoIter::from(self)
    }
}
