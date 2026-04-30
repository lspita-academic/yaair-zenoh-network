use std::str::FromStr;

use zenoh_pico_core::{
    result::{IntoZenohResult, ZenohError},
    sys::{_z_declared_keyexpr_t, z_keyexpr_from_substr_autocanonize},
    zvalue::{ZOwn, ZValue},
};
use zenoh_pico_macros::zwrap;

#[zwrap(base(name = "keyexpr"), zvalue(value_ty = _z_declared_keyexpr_t), zown)]
pub struct KeyExpr;

impl FromStr for KeyExpr {
    type Err = ZenohError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut value = Self::uninitialized();
        value
            .inspect_zowned_mut(|z| unsafe {
                z_keyexpr_from_substr_autocanonize(z, s.as_ptr(), &mut s.len()).into_zresult()
            })
            .map(|_| value)
    }
}
