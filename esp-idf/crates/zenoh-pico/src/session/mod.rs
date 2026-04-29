pub mod config;
pub mod locator;
pub mod publisher;
pub mod subscriber;
pub mod whatami;

use zenoh_pico_core::{
    sys::{
        z_close, z_open, z_open_options_default, z_open_options_t, z_owned_session_t,
        z_publisher_options_t, z_session_is_closed, z_session_loan_mut, zp_start_lease_task,
        zp_start_read_task,
    },
    zvalue::{ZLoan, ZLoanMut, ZOwn},
};
use zenoh_pico_macros::zown;

use crate::{
    keyexpr::KeyExpr,
    options::{ZDefaultFn, ZOptionsDefault},
    result::{IntoZenohResult, ZenohResult},
    session::{config::ZenohConfig, publisher::Publisher, subscriber::Subscriber},
};

impl ZDefaultFn for z_open_options_t {
    fn zdefault_fn() -> unsafe extern "C" fn(*mut Self) {
        z_open_options_default
    }
}

#[zown(base = "session", zloan(mutable))]
pub struct Session;

impl Session {
    pub fn open(config: ZenohConfig, open_options: Option<z_open_options_t>) -> ZenohResult<Self> {
        let open_options = open_options.unwrap_or_else(ZOptionsDefault::options_default);
        let mut session = z_owned_session_t::default();
        unsafe {
            z_open(&mut session, config.zmove(), &open_options).into_zresult()?;
        }
        unsafe {
            // not done automatically, even if it should be because of the default options
            zp_start_read_task(z_session_loan_mut(&mut session), std::ptr::null());
            zp_start_lease_task(z_session_loan_mut(&mut session), std::ptr::null());
        }

        Ok(Self::from(session))
    }

    pub fn close(mut self) {
        let session_closed = unsafe { z_session_is_closed(self.zloan()) };
        if session_closed {
            return;
        }
        let close_options = std::ptr::null(); // dummy struct
        unsafe {
            z_close(self.zloan_mut(), close_options);
        }
    }

    pub fn publisher(
        &self,
        key: &KeyExpr,
        publisher_options: Option<z_publisher_options_t>,
    ) -> ZenohResult<Publisher> {
        Publisher::declare(self, key, publisher_options)
    }

    pub fn subscriber(&self, key: &KeyExpr) -> ZenohResult<Subscriber> {
        Subscriber::declare(self, key)
    }
}
