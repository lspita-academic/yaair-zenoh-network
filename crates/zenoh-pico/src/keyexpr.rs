use std::str::FromStr;

use zenoh_pico_macros::zwrap;

use crate::{
    result::{IntoZenohResult, ZenohError, ZenohResult},
    session::Session,
    sys::{
        _z_declared_keyexpr_t, z_declare_keyexpr, z_keyexpr_equals,
        z_keyexpr_from_substr_autocanonize,
    },
    zvalue::{ZOwn, ZValue},
};

#[zwrap(base(name = "keyexpr"), zvalue(value_ty = _z_declared_keyexpr_t), zown)]
pub struct KeyExpr;

impl KeyExpr {
    pub fn declare(self, session: &Session) -> ZenohResult<Self> {
        let mut keyexpr = Self::uninitialized();
        keyexpr
            .with_zowned_mut(|z| unsafe {
                z_declare_keyexpr(session.zloan(), z, self.zloan()).into_zresult()
            })
            .map(|_| keyexpr)
    }
}

impl FromStr for KeyExpr {
    type Err = ZenohError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut keyexpr = Self::uninitialized();
        keyexpr
            .with_zowned_mut(|z| unsafe {
                z_keyexpr_from_substr_autocanonize(z, s.as_ptr(), &mut s.len()).into_zresult()
            })
            .map(|_| keyexpr)
    }
}

impl Eq for KeyExpr {}
impl PartialEq for KeyExpr {
    fn eq(&self, other: &Self) -> bool {
        unsafe { z_keyexpr_equals(self.zloan(), other.zloan()) }
    }
}
