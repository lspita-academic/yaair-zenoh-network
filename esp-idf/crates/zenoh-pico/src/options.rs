pub trait ZDefaultFn: Default {
    fn zdefault_fn() -> unsafe extern "C" fn(*mut Self);
}

pub trait ZOptionsDefault: ZDefaultFn {
    fn options_default() -> Self;
}

impl<T: ZDefaultFn> ZOptionsDefault for T {
    fn options_default() -> Self {
        let mut value = T::default();
        unsafe {
            T::zdefault_fn()(&mut value);
        }
        value
    }
}
