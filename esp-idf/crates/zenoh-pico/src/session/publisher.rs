use zenoh_pico_core::{
    result::{IntoZenohResult, ZenohResult},
    sys::{
        z_declare_publisher, z_publisher_options_default, z_publisher_options_t,
        z_undeclare_publisher,
    },
    zvalue::ZLoan,
};
use zenoh_pico_macros::zown;

use crate::{
    keyexpr::KeyExpr,
    options::{ZDefaultFn, ZOptionsDefault},
    session::Session,
};

impl ZDefaultFn for z_publisher_options_t {
    fn zdefault_fn() -> unsafe extern "C" fn(*mut Self) {
        z_publisher_options_default
    }
}

#[zown(base = "publisher", zmove(drop_zfn = z_undeclare_publisher), zloan(mutable))]
pub struct Publisher;

impl Publisher {
    pub fn declare(
        session: &Session,
        key: &KeyExpr,
        publisher_options: Option<z_publisher_options_t>,
    ) -> ZenohResult<Self> {
        let publisher_options = publisher_options.unwrap_or_else(ZOptionsDefault::options_default);
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
