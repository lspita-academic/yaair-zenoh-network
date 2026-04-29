use std::ptr::NonNull;

use zenoh_pico_core::{
    sys::{
        z_sample_attachment, z_sample_kind, z_sample_kind_t, z_sample_kind_t_Z_SAMPLE_KIND_DELETE, z_sample_kind_t_Z_SAMPLE_KIND_PUT, z_sample_timestamp
    },
    zvalue::{ZLoan, ZValue},
};
use zenoh_pico_macros::{zclosure, zown};

use crate::{timestamp::Timestamp, zbytes::ZBytes};

#[zown(base = "sample", zloan(mutable), ztake)]
pub struct Sample;

#[zclosure(base = "sample", zloan)]
pub struct SampleClosure;

pub enum SampleKind {
    PUT,
    DELETE,
}

impl Sample {
    pub fn kind(&self) -> SampleKind {
        let zkind: z_sample_kind_t = unsafe { z_sample_kind(self.zloan()) };
        match zkind {
            #![allow(non_upper_case_globals)]
            z_sample_kind_t_Z_SAMPLE_KIND_PUT => SampleKind::PUT,
            z_sample_kind_t_Z_SAMPLE_KIND_DELETE => SampleKind::DELETE,
            _ => unreachable!("bindgen converts enum to generic u32"),
        }
    }

    pub fn timestamp(&self) -> Option<&Timestamp> {
        NonNull::new(unsafe { z_sample_timestamp(self.zloan()) } as *mut _)
            .map(|nn| unsafe { Timestamp::from_raw(nn.as_ptr()) })
    }

    pub fn attachment(&self) -> Option<&ZBytes> {
        NonNull::new(unsafe { z_sample_attachment(self.zloan()) } as *mut _)
            .map(|nn| unsafe { ZBytes::from_raw(nn.as_ptr()) })
    }
}
