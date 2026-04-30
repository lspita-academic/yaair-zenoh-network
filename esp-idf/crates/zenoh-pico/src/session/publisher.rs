use zenoh_pico_core::{
    result::{IntoZenohResult, ZenohResult},
    sys::{
        z_declare_publisher, z_publisher_options_default, z_publisher_options_t,
        z_undeclare_publisher,
    },
    zvalue::ZValue,
};
use zenoh_pico_macros::zwrap;

use crate::{
    keyexpr::KeyExpr,
    session::Session,
    zoptions::{ZOptionsDefault, ZOptionsInit},
};

impl ZOptionsInit for z_publisher_options_t {
    fn zinit(&mut self) {
        unsafe {
            z_publisher_options_default(self);
        }
    }
}

#[zwrap(base(name = "publisher"), zvalue, zown(drop_zfn = z_undeclare_publisher))]
pub struct Publisher;

impl Publisher {
    pub fn declare(
        session: &Session,
        key: &KeyExpr,
        publisher_options: Option<z_publisher_options_t>,
    ) -> ZenohResult<Self> {
        let publisher_options = publisher_options.unwrap_or_else(ZOptionsDefault::zdefault);
        let mut publisher = Default::default();
        unsafe {
            z_declare_publisher(
                session.zloan(),
                &mut publisher,
                key.zloan(),
                &publisher_options,
            )
            .into_zresult()?;
        };

        Ok(Self::from(publisher))
    }
}
