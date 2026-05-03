use zenoh_pico_macros::zwrap;
use zenoh_pico_sys::{
    z_publisher_keyexpr, z_publisher_options_default,
    z_publisher_options_t, z_publisher_put, z_publisher_put_options_default,
    z_publisher_put_options_t, z_undeclare_publisher,
};

use crate::{
    keyexpr::KeyExpr,
    result::{IntoZenohResult, ZenohResult},
    zbytes::IntoZBytes,
    zoptions::{ZOptionsInit, options_ptr},
    zvalue::{ZOwn, ZValue},
};

impl ZOptionsInit for z_publisher_options_t {
    fn zinit(&mut self) {
        unsafe {
            z_publisher_options_default(self);
        }
    }
}

impl ZOptionsInit for z_publisher_put_options_t {
    fn zinit(&mut self) {
        unsafe {
            z_publisher_put_options_default(self);
        }
    }
}

#[zwrap(base(name = "publisher"), zvalue, zown(drop_zfn = z_undeclare_publisher))]
pub struct Publisher;

impl Publisher {
    pub fn put<V: IntoZBytes>(
        &self,
        value: V,
        put_options: Option<z_publisher_put_options_t>,
    ) -> ZenohResult<()> {
        let put_options = options_ptr(put_options.as_ref());
        let payload = value.into_zbytes();
        unsafe { z_publisher_put(self.zloan(), &mut payload.zmove(), put_options).into_zresult() }
    }

    pub fn keyexpr(&self) -> &KeyExpr {
        KeyExpr::from_ptr(unsafe { z_publisher_keyexpr(self.zloan()) })
    }
}
