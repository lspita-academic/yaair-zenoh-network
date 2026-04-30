use std::{error::Error, ffi::c_void, fmt::Debug};

use embassy_sync::{blocking_mutex::raw::RawMutex, signal::Signal};
use zenoh_pico_sys::z_closure_drop_callback_t;

use crate::result::ZenohResult;

pub trait CType: Default + Debug + Copy + Clone + Sized {}
impl<T: Default + Debug + Copy + Clone> CType for T {}

pub trait ZValue: Sized {
    type Value: CType;

    fn uninitialized() -> Self;
    fn from_zvalue(value: Self::Value) -> Self;
    fn from_raw<'a>(ptr: *const Self::Value) -> &'a Self;
    fn from_raw_mut<'a>(ptr: *mut Self::Value) -> &'a mut Self;
    fn zloan(&self) -> *const Self::Value;
    fn zloan_mut(&mut self) -> *mut Self::Value;
}

pub trait ZOwn: ZValue {
    type OwnedValue: CType;
    type MovedValue: CType;

    fn inspect_zowned<F, T>(&self, f: F) -> T
    where
        F: FnOnce(&Self::OwnedValue) -> T;

    fn inspect_zowned_mut<F, T>(&mut self, f: F) -> T
    where
        F: FnOnce(&mut Self::OwnedValue) -> T;
    fn from_zowned(value: Self::OwnedValue) -> Self;
    fn zmove(self) -> *mut Self::MovedValue;
    fn zdrop(&mut self);
}

pub trait ZTake: ZOwn {
    type Error: Error;

    fn ztake(loan_mut: *mut Self::Value) -> Result<Self, Self::Error>;
}

pub trait ZClosure: ZOwn {
    type CallbackValue: CType;

    fn from_callback<T>(
        callback: unsafe extern "C" fn(*mut Self::CallbackValue, *mut c_void),
        drop: z_closure_drop_callback_t,
        context: Option<&mut T>,
    ) -> ZenohResult<Self>;

    fn from_signal<M: RawMutex, T: ZTake<Value = Self::CallbackValue>>(
        signal: &mut Signal<M, Result<T, T::Error>>,
        drop: z_closure_drop_callback_t,
    ) -> ZenohResult<Self> {
        Self::from_callback(zclosure_signal_callback::<M, T>, drop, Some(signal))
    }
}

unsafe extern "C" fn zclosure_signal_callback<M: RawMutex, T: ZTake>(
    value: *mut T::Value,
    context: *mut c_void,
) {
    let signal = unsafe { &mut *(context as *mut Signal<M, Result<T, T::Error>>) };
    let value = T::ztake(value);
    signal.signal(value);
}
