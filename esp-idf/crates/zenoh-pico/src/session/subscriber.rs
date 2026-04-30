use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, signal::Signal};
use zenoh_pico_core::{
    result::{IntoZenohResult, ZenohResult},
    sys::{z_declare_subscriber, z_undeclare_subscriber},
    zvalue::{ZClosure, ZOwn, ZValue},
};
use zenoh_pico_macros::zwrap;

use crate::{
    keyexpr::KeyExpr,
    sample::{Sample, SampleClosure},
    session::Session,
};

#[zwrap(base(name = "subscriber"), zvalue, zown(drop_zfn = z_undeclare_subscriber))]
struct InternalSubscriber;

pub struct Subscriber {
    /// The internal subscriber is stored to keep it alive the entire lifetime
    /// and call undeclare only on drop.
    /// The alternative would be to undeclare on drop of this struct, but since
    /// fields cannot be moved out and consumed because of the shared reference,
    /// it would require reimplementing what the zown macro does manually.
    _subscriber: InternalSubscriber,
    signal: Signal<CriticalSectionRawMutex, ZenohResult<Sample>>,
}

impl Subscriber {
    pub fn declare(session: &Session, key: &KeyExpr) -> ZenohResult<Self> {
        let mut signal = Signal::new();
        let sample_closure = SampleClosure::from_signal(&mut signal, None)?;

        let subscriber_options = std::ptr::null(); // dummy struct
        let mut subscriber = Default::default();
        unsafe {
            z_declare_subscriber(
                session.zloan(),
                &mut subscriber,
                key.zloan(),
                sample_closure.zmove(),
                subscriber_options,
            )
            .into_zresult()?;
        };
        let subscriber = InternalSubscriber::from(subscriber);

        Ok(Self {
            _subscriber: subscriber,
            signal,
        })
    }

    pub async fn recv_async(&self) -> ZenohResult<Sample> {
        self.signal.wait().await
    }
}
