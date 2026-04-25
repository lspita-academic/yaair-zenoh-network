use std::fmt::Debug;

pub trait ZValue<T: Default>: From<T> + Default + Debug {}

pub trait ZOwn<T: Default, M>: ZValue<T> {
    fn zmove(self) -> *mut M;
}

pub trait ZView<T: Default>: ZValue<T> {}

pub trait ZLoan<T: Default, L: Default>: ZValue<T> {
    fn zloan(&self) -> *const L;
}

pub trait ZLoanMut<T: Default, L: Default>: ZValue<T> {
    fn zloan_mut(&mut self) -> *mut L;
}
