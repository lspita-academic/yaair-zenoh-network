use std::ptr::NonNull;

pub trait NonNullExtensions<T> {
    fn from_ptr(ptr: *const T) -> Option<NonNull<T>>;
    fn from_ptr_mut(ptr: *mut T) -> Option<NonNull<T>>;
}

impl<T> NonNullExtensions<T> for NonNull<T> {
    fn from_ptr(ptr: *const T) -> Option<NonNull<T>> {
        Self::new(ptr as *mut _)
    }

    fn from_ptr_mut(ptr: *mut T) -> Option<NonNull<T>> {
        Self::new(ptr)
    }
}
