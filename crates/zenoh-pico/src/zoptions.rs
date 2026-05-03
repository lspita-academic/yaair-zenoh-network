use crate::zvalue::CType;

pub trait ZOptionsDefault {
    fn zdefault() -> Self;
}

pub trait ZOptionsInit {
    fn zinit(&mut self);
}

impl<T: ZOptionsInit + CType> ZOptionsDefault for T {
    fn zdefault() -> Self {
        let mut value = Self::default();
        value.zinit();
        value
    }
}

pub fn options_ptr<T: ZOptionsDefault>(opt: Option<&T>) -> *const T {
    opt.map(|o| o as *const _).unwrap_or(core::ptr::null())
}
