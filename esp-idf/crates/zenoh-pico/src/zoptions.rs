use zenoh_pico_core::zvalue::CType;

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
