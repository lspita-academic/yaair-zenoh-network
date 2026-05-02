use std::str::FromStr;

use zenoh_pico_core::{
    result::{IntoZenohResult, ZenohError, ZenohResult},
    sys::{_z_declared_keyexpr_t, z_declare_keyexpr, z_keyexpr_from_substr_autocanonize},
    zvalue::{ZOwn, ZValue},
};
use zenoh_pico_macros::zwrap;

use crate::session::Session;

#[zwrap(base(name = "keyexpr"), zvalue(value_ty = _z_declared_keyexpr_t), zown)]
pub struct KeyExpr;

impl KeyExpr {
    pub fn declare(self, session: &Session) -> ZenohResult<Self> {
        let mut value = Self::uninitialized();
        value
            .inspect_zowned_mut(|z| unsafe {
                z_declare_keyexpr(session.zloan(), z, self.zloan()).into_zresult()
            })
            .map(|_| value)
    }
}

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
