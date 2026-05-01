use std::{ffi::c_void, fmt::Debug, sync::Arc};

use embassy_sync::{blocking_mutex::raw::RawMutex, signal::Signal};

use crate::result::ZenohResult;

pub trait CType: Default + Debug + Copy + Clone + Sized {}
impl<T: Default + Debug + Copy + Clone> CType for T {}

pub trait ZValue
where
    Self: Sized,
{
    type Value: CType;

    fn uninitialized() -> Self;
    fn from_zvalue(value: Self::Value) -> Self;
    fn from_ptr<'a>(ptr: *const Self::Value) -> &'a Self;
    fn from_ptr_mut<'a>(ptr: *mut Self::Value) -> &'a mut Self;
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

pub trait ZClone: ZOwn {
    fn zclone(loan: *const Self::Value) -> Self;
}

pub trait ZClosure: ZOwn {
    type CallbackValue: CType;

    fn from_callback<T>(
        callback: unsafe extern "C" fn(*mut Self::CallbackValue, *mut c_void),
        context: Option<Arc<T>>,
    ) -> ZenohResult<Self>;

    fn from_signal<M: RawMutex, T: ZValue<Value = Self::CallbackValue>>(
        signal: Arc<Signal<M, T>>,
    ) -> ZenohResult<Self> {
        unsafe extern "C" fn zclosure_signal_callback<M: RawMutex, T: ZValue>(
            value: *mut T::Value,
            context: *mut c_void,
        ) {
            let signal = unsafe { &mut *(context as *mut Signal<M, T>) };
            let value = T::from_zvalue(unsafe { *value });
            signal.signal(value);
        }

        Self::from_callback(zclosure_signal_callback::<M, T>, Some(signal))
    }
}
