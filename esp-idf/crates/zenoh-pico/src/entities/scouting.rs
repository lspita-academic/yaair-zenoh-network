use num_enum::UnsafeFromPrimitive;
use zenoh_pico_core::{
    sys::{z_hello_whatami, z_hello_zid, zp_hello_locators},
    zvalue::ZValue,
};
use zenoh_pico_macros::zwrap;

use crate::{
    entities::{locator::Locator, whatami::WhatAmI}, zid::ZId, zstring::ZStringArray
};

#[zwrap(base(name = "hello"), zvalue, zown)]
pub struct ScoutHello;

impl ScoutHello {
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
