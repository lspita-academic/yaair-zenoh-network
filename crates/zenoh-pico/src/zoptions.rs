use crate::zvalue::CType;

pub trait ZOptions {
    fn zdefault() -> Self;
}

pub trait ZOptionsInit {
    fn zinit(&mut self);
}

impl<T: ZOptionsInit + CType> ZOptions for T {
    fn zdefault() -> Self {
        let mut value = Self::default();
        value.zinit();
        value
    }
}

pub fn options_ptr<T: ZOptions>(opt: Option<&T>) -> *const T {
    opt.map(|o| o as *const _).unwrap_or(core::ptr::null())
}

pub fn options_ptr_mut<T: ZOptions>(opt: Option<&mut T>) -> *mut T {
    opt.map(|o| o as *mut _).unwrap_or(core::ptr::null_mut())
}
