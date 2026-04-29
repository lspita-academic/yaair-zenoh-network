use uhlc::NTP64;
use zenoh_pico_core::sys::{z_timestamp_id, z_timestamp_ntp64_time};
use zenoh_pico_macros::zwrap;

use crate::zid::ZId;

#[zwrap(base = "timestamp")]
pub struct Timestamp;

impl Timestamp {
    pub fn zid(&self) -> ZId {
        ZId::from(unsafe { z_timestamp_id(&self.0) })
    }

    pub fn ntp64(&self) -> NTP64 {
        NTP64(unsafe { z_timestamp_ntp64_time(&self.0) })
    }
}
