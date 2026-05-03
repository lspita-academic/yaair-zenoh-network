pub mod publisher;
pub mod subscriber;

use std::sync::Arc;

use embassy_sync::signal::Signal;
use zenoh_pico_macros::zwrap;

use crate::{
    config::ZenohConfig,
    keyexpr::KeyExpr,
    result::{IntoZenohResult, ZenohResult},
    sample::SampleClosure,
    session::{
        publisher::Publisher,
        subscriber::{InternalSubscriber, Subscriber},
    },
    sys::{
        z_close, z_close_options_t, z_declare_publisher, z_declare_subscriber, z_open,
        z_open_options_default, z_open_options_t, z_publisher_options_t, z_session_is_closed,
        z_subscriber_options_t, zp_start_lease_task, zp_start_read_task,
    },
    zoptions::{ZOptionsInit, options_ptr},
    zvalue::{ZClosure, ZOwn, ZValue},
};

impl ZOptionsInit for z_open_options_t {
    fn zinit(&mut self) {
        unsafe {
            z_open_options_default(self);
        }
    }
}

impl ZOptionsInit for z_close_options_t {
    fn zinit(&mut self) {
        // dummy struct
    }
}

#[zwrap(base(name = "session", family = "rc"), zvalue, zown)]
pub struct Session;

impl Session {
    pub fn open(config: ZenohConfig, open_options: Option<z_open_options_t>) -> ZenohResult<Self> {
        let open_options = options_ptr(open_options.as_ref());
        let mut session = Self::uninitialized();
        session
            .with_zowned_mut(|z| unsafe {
                z_open(z, &mut config.zmove(), open_options).into_zresult()
            })
            // not done automatically, even if it should be because of the default options
            .and_then(|_| unsafe {
                zp_start_read_task(session.zloan_mut(), std::ptr::null()).into_zresult()
            })
            .and_then(|_| unsafe {
                zp_start_lease_task(session.zloan_mut(), std::ptr::null()).into_zresult()
            })
            .map(|_| session)
    }

    pub fn close(mut self, close_options: Option<z_close_options_t>) {
        let session_closed = unsafe { z_session_is_closed(self.zloan()) };
        if session_closed {
            return;
        }
        let close_options = options_ptr(close_options.as_ref());
        unsafe {
            z_close(self.zloan_mut(), close_options);
        }
    }

    pub fn declare_publisher(
        &self,
        key: &KeyExpr,
        publisher_options: Option<z_publisher_options_t>,
    ) -> ZenohResult<Publisher> {
        let publisher_options = options_ptr(publisher_options.as_ref());
        let mut publisher = Publisher::uninitialized();
        publisher.with_zowned_mut(|z| unsafe {
            z_declare_publisher(self.zloan(), z, key.zloan(), publisher_options).into_zresult()
        })?;

        Ok(publisher)
    }

    pub fn declare_subscriber(
        &self,
        key: &KeyExpr,
        subscriber_options: Option<z_subscriber_options_t>,
    ) -> ZenohResult<Subscriber> {
        let signal = Arc::new(Signal::new());
        let sample_closure = SampleClosure::from_signal(signal.clone())?;

        let subscriber_options = options_ptr(subscriber_options.as_ref());
        let mut subscriber = InternalSubscriber::uninitialized();
        subscriber.with_zowned_mut(|z| unsafe {
            z_declare_subscriber(
                self.zloan(),
                z,
                key.zloan(),
                &mut sample_closure.zmove(),
                subscriber_options,
            )
            .into_zresult()
        })?;

        Ok(Subscriber { subscriber, signal })
    }
}
