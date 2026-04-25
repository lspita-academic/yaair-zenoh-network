use std::ffi::CString;

pub trait CStringExtensions {
    fn from_vec_maybe_nul(value: impl Into<Vec<u8>>) -> Self;
}

impl CStringExtensions for CString {
    fn from_vec_maybe_nul(value: impl Into<Vec<u8>>) -> Self {
        let value = value
            .into()
            .into_iter()
            .take_while(|b| *b != 0)
            .collect::<Vec<_>>();
        unsafe { Self::from_vec_unchecked(value) }
    }
}
