use std::fmt::Debug;

use crate::{result::ZenohResult, sys::z_closure_drop_callback_t};

pub trait CType: Default + Debug {}
impl<T: Default + Debug> CType for T {}

pub trait ZValue: From<Self::Value> + Debug {
    type Value: CType;
}

pub trait ZOwn: ZValue {
    type MovedValue: CType;

    fn zmove(self) -> *mut Self::MovedValue;
}

pub trait ZView: ZValue {}

pub trait ZClosure: ZOwn {
    type CallbackFn;

    fn from_callback<T>(
        callback: Self::CallbackFn,
        drop: z_closure_drop_callback_t,
        context: Option<&mut T>,
    ) -> ZenohResult<Self>;
}

pub trait ZLoan: ZValue {
    type LoanedValue: CType;

    fn zloan(&self) -> *const Self::LoanedValue;
}

pub trait ZLoanMut: ZLoan {
    fn zloan_mut(&mut self) -> *mut <Self as ZLoan>::LoanedValue;
}
