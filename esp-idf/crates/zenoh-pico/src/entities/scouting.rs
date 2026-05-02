use std::sync::Arc;

use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, signal::Signal};
use num_enum::UnsafeFromPrimitive;
use zenoh_pico_core::{
    result::{IntoZenohResult, ZenohResult},
    sys::{
        z_hello_whatami, z_hello_zid, z_scout, z_scout_options_default, z_scout_options_t,
        zp_hello_locators,
    },
    zvalue::{ZClosure, ZOwn, ZValue},
};
use zenoh_pico_macros::zwrap;

use crate::{
    config::ZenohConfig,
    entities::{locator::Locator, whatami::WhatAmI},
    zid::ZId,
    zoptions::{ZOptionsInit, options_ptr},
    zstring::ZStringArray,
};

#[zwrap(base(name = "hello"), zvalue, zown)]
pub struct Hello;

#[zwrap(base(name = "closure_hello"), zvalue, zown, zclosure)]
pub struct HelloClosure;

impl Hello {
    pub fn whatami(&self) -> WhatAmI {
        unsafe { WhatAmI::unchecked_transmute_from(z_hello_whatami(self.zloan())) }
    }

    pub fn locators(&self) -> impl Iterator<Item = Locator> {
        let locators_zstrings = ZStringArray::from_ptr(unsafe { zp_hello_locators(self.zloan()) });
        locators_zstrings.into_iter().map(|s| {
            s.parse()
                .expect("locators array should contain valid locators")
        })
    }

    pub fn zid(&self) -> ZId {
        unsafe { z_hello_zid(self.zloan()) }.into()
    }
}

impl ZOptionsInit for z_scout_options_t {
    fn zinit(&mut self) {
        unsafe {
            z_scout_options_default(self);
        }
    }
}

pub struct Scout {
    signal: Arc<Signal<CriticalSectionRawMutex, Hello>>,
}

impl Scout {
    pub fn start(
        config: ZenohConfig,
        scout_options: Option<z_scout_options_t>,
    ) -> ZenohResult<Self> {
        let scout_options = options_ptr(scout_options.as_ref());
        let signal = Arc::new(Signal::new());
        let closure = HelloClosure::from_signal(signal.clone())?;

        unsafe {
            z_scout(config.zmove(), closure.zmove(), scout_options).into_zresult()?;
        }
        Ok(Self { signal })
    }

    pub async fn recv_async(&self) -> Hello {
        self.signal.wait().await
    }
}
