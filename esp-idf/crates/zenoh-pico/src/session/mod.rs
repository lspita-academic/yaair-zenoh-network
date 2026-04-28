pub mod publisher;
pub mod subscriber;

use zenoh_pico_core::{
    sys::{
        z_close, z_open, z_open_options_default, z_open_options_t, z_owned_session_t,
        z_session_is_closed, z_session_loan_mut, zp_start_lease_task, zp_start_read_task,
    },
    zvalue::{ZLoan, ZLoanMut, ZOwn},
};
use zenoh_pico_macros::zown;

use crate::{
    config::ZenohConfig,
    keyexpr::KeyExpr,
    options::{ZDefaultFn, ZOptionsDefault},
    result::{IntoZenohResult, ZenohResult},
    session::subscriber::Subscriber,
};

impl ZDefaultFn for z_open_options_t {
    fn zdefault_fn() -> unsafe extern "C" fn(*mut Self) {
        z_open_options_default
    }
}

#[zown(base = "session", zloan(mutable))]
pub struct ZenohSession;

impl ZenohSession {
    pub fn open(config: ZenohConfig, open_options: Option<z_open_options_t>) -> ZenohResult<Self> {
        let mut zsession = z_owned_session_t::default();
        let open_options = open_options.unwrap_or_else(ZOptionsDefault::options_default);
        unsafe {
            z_open(&mut zsession, config.zmove(), &open_options).into_zresult()?;
        }
        unsafe {
            // not done automatically, even if it should be because of the default options
            zp_start_read_task(z_session_loan_mut(&mut zsession), std::ptr::null());
            zp_start_lease_task(z_session_loan_mut(&mut zsession), std::ptr::null());
        }

        Ok(Self::from(zsession))
    }

    pub fn close(mut self) {
        let session_closed = unsafe { z_session_is_closed(self.zloan()) };
        if !session_closed {
            return;
        }
        let close_options = std::ptr::null(); // dummy struct
        unsafe {
            z_close(self.zloan_mut(), close_options);
        }
    }

    pub fn publisher(&self, key: &str) -> ZenohPublisher {
        ZenohPublisher::new(self, key)
    }

    pub fn subscriber(&self, key: &KeyExpr) -> ZenohResult<Subscriber> {
        Subscriber::declare(self, key)
    }
}
