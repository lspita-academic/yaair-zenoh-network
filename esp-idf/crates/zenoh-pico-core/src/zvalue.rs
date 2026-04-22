use std::fmt::Debug;

pub trait ZValue<T: Default, M>: From<T> + Default + Debug {
    fn zmove(self) -> *mut M;
}

pub trait ZLoan<T: Default, M, L: Default>: ZValue<T, M> {
    fn zloan(&self) -> *const L;
}

pub trait ZLoanMut<T: Default, M, L: Default>: ZValue<T, M> {
    fn zloan_mut(&mut self) -> *mut L;
}
