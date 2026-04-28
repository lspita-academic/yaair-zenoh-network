use std::{ffi::c_void, fmt::Debug};

use embassy_sync::{blocking_mutex::raw::RawMutex, signal::Signal};

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

pub trait ZLoan: ZValue {
    type LoanedValue: CType;

    fn zloan(&self) -> *const Self::LoanedValue;
}

pub trait ZLoanMut: ZLoan {
    fn zloan_mut(&mut self) -> *mut <Self as ZLoan>::LoanedValue;
}

pub trait ZTake: ZOwn + ZLoanMut + TryFrom<*mut <Self as ZLoan>::LoanedValue> {}

pub trait ZClosure: ZOwn {
    type CallbackValue: CType;

    fn from_callback<T>(
        callback: unsafe extern "C" fn(*mut Self::CallbackValue, *mut c_void),
        drop: z_closure_drop_callback_t,
        context: Option<&mut T>,
    ) -> ZenohResult<Self>;

    fn from_signal<M: RawMutex, T: ZTake<LoanedValue = Self::CallbackValue>>(
        signal: &mut Signal<M, Result<T, T::Error>>,
        drop: z_closure_drop_callback_t,
    ) -> ZenohResult<Self> {
        Self::from_callback(
            zclosure_signal_callback::<M, T>,
            drop,
            Some(signal),
        )
    }
}

unsafe extern "C" fn zclosure_signal_callback<M: RawMutex, T: ZTake>(
    value: *mut T::LoanedValue,
    context: *mut c_void,
) {
    let signal = unsafe { &mut *(context as *mut Signal<M, Result<T, T::Error>>) };
    let value = T::try_from(value);
    signal.signal(value);
}
