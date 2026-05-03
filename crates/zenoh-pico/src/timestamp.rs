use uhlc::NTP64;
use zenoh_pico_macros::zwrap;

use crate::{
    sys::{z_timestamp_id, z_timestamp_ntp64_time},
    zid::ZId,
    zvalue::ZValue,
};

#[zwrap(base(name = "timestamp"), zvalue)]
pub struct Timestamp;

impl Timestamp {
    pub fn zid(&self) -> ZId {
        ZId::from(unsafe { z_timestamp_id(self.zloan()) })
    }

    pub fn ntp64(&self) -> NTP64 {
        NTP64(unsafe { z_timestamp_ntp64_time(self.zloan()) })
    }
}
