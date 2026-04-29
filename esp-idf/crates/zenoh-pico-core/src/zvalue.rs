use std::{ffi::c_void, fmt::Debug, ops::DerefMut};

use embassy_sync::{blocking_mutex::raw::RawMutex, signal::Signal};
use zenoh_pico_sys::z_closure_drop_callback_t;

use crate::result::ZenohResult;

pub trait CType: Default + Debug {}
impl<T: Default + Debug> CType for T {}

pub trait ZValue: From<Self::Value> + Debug + DerefMut {
    type Value: CType;

    unsafe fn from_raw<'a>(ptr: *const Self::Value) -> &'a Self;
    unsafe fn from_raw_mut<'a>(ptr: *mut Self::Value) -> &'a mut Self;

    fn try_initialize<I, E>(initializer: I) -> Result<Self, E>
    where
        I: FnOnce(&mut Self::Value) -> Result<(), E>,
    {
        let mut zval = Self::Value::default();
        initializer(&mut zval)?;
        Ok(Self::from(zval))
    }

    fn initialize<I>(initializer: I) -> Self
    where
        I: FnOnce(&mut Self::Value),
    {
        Self::try_initialize(|value| {
            initializer(value);
            Result::<_, ()>::Ok(())
        })
        .unwrap()
    }
}

pub trait ZOwn: ZValue {
    type OwnedValue: CType;
    type MovedValue: CType;

    fn zowned(&mut self) -> *mut Self::OwnedValue;
    fn zmove(self) -> *mut Self::MovedValue;
}

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
        Self::from_callback(zclosure_signal_callback::<M, T>, drop, Some(signal))
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
