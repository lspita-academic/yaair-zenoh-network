use std::sync::Arc;

use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, signal::Signal};
use zenoh_pico_macros::zwrap;
use zenoh_pico_sys::{
    z_publisher_delete, z_publisher_delete_options_default, z_publisher_delete_options_t,
    z_publisher_get_matching_status, z_publisher_keyexpr, z_publisher_options_default,
    z_publisher_options_t, z_publisher_put, z_publisher_put_options_default,
    z_publisher_put_options_t, z_subscriber_keyexpr, z_subscriber_options_t, z_undeclare_publisher,
    z_undeclare_subscriber,
};

use crate::{
    keyexpr::KeyExpr,
    result::{IntoZenohResult, ZenohResult},
    sample::Sample,
    session::matching::MatchingStatus,
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

impl ZOptionsInit for z_publisher_delete_options_t {
    fn zinit(&mut self) {
        unsafe {
            z_publisher_delete_options_default(self);
        }
    }
}

#[zwrap(base(name = "publisher"), zvalue, zown(drop_zfn = z_undeclare_publisher))]
pub struct Publisher;

impl Publisher {
    pub fn put<Payload: IntoZBytes>(
        &self,
        payload: Payload,
        options: Option<z_publisher_put_options_t>,
    ) -> ZenohResult<()> {
        let options = options_ptr(options.as_ref());
        let payload = payload.into_zbytes();
        unsafe { z_publisher_put(self.zloan(), &mut payload.zmove(), options).into_zresult() }
    }

    pub fn delete(&self, options: Option<z_publisher_delete_options_t>) -> ZenohResult<()> {
        let options = options_ptr(options.as_ref());
        unsafe { z_publisher_delete(self.zloan(), options).into_zresult() }
    }

    pub fn matching_status(&self) -> ZenohResult<MatchingStatus> {
        let mut matching_status = MatchingStatus::uninitialized();
        unsafe {
            z_publisher_get_matching_status(self.zloan(), matching_status.zloan_mut())
                .into_zresult()
        }?;
        Ok(matching_status)
    }

    pub fn keyexpr(&self) -> &KeyExpr {
        KeyExpr::from_ptr(unsafe { z_publisher_keyexpr(self.zloan()) })
    }
}

#[zwrap(base(name = "subscriber"), zvalue, zown(drop_zfn = z_undeclare_subscriber))]
pub(super) struct InternalSubscriber;

pub struct Subscriber {
    pub(super) subscriber: InternalSubscriber,
    pub(super) signal: Arc<Signal<CriticalSectionRawMutex, Sample>>,
}

impl ZOptionsInit for z_subscriber_options_t {
    fn zinit(&mut self) {
        // dummy struct
    }
}

impl Subscriber {
    pub async fn recv_async(&self) -> Sample {
        self.signal.wait().await
    }

    pub fn keyexpr(&self) -> &KeyExpr {
        KeyExpr::from_ptr(unsafe { z_subscriber_keyexpr(self.subscriber.zloan()) })
    }
}
