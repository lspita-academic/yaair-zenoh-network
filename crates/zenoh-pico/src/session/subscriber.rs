use std::sync::Arc;

use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, signal::Signal};
use zenoh_pico_macros::zwrap;
use zenoh_pico_sys::{z_subscriber_keyexpr, z_subscriber_options_t, z_undeclare_subscriber};

use crate::{keyexpr::KeyExpr, sample::Sample, zoptions::ZOptionsInit, zvalue::ZValue};

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
