pub use zenoh_pico_macros::ZValue;

pub trait ZValue<T>: Into<T> + From<T> {
    fn zvalue(&self) -> &T;
    fn zvalue_mut(&mut self) -> &mut T;
}
