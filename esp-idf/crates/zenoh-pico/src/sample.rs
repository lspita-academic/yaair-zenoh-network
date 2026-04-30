use std::ptr::NonNull;

use ffi_utils::pointer::NonNullExtensions;
use zenoh_pico_core::{
    sys::{
        z_sample_attachment, z_sample_encoding, z_sample_keyexpr, z_sample_kind, z_sample_kind_t, z_sample_kind_t_Z_SAMPLE_KIND_DELETE, z_sample_kind_t_Z_SAMPLE_KIND_PUT, z_sample_payload, z_sample_timestamp
    },
    zvalue::ZValue,
};
use zenoh_pico_macros::zwrap;

use crate::{encoding::Encoding, keyexpr::KeyExpr, timestamp::Timestamp, zbytes::ZBytes};

#[zwrap(base(name = "sample"), zvalue, zown, ztake)]
pub struct Sample;

#[zwrap(base(name = "closure_sample"), zvalue, zown, zclosure)]
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
        NonNull::from_ptr(unsafe { z_sample_timestamp(self.zloan()) })
            .map(|nn| Timestamp::from_ptr(nn.as_ptr()))
    }

    pub fn attachment(&self) -> Option<&ZBytes> {
        NonNull::from_ptr(unsafe { z_sample_attachment(self.zloan()) })
            .map(|nn| ZBytes::from_ptr(nn.as_ptr()))
    }

    pub fn encoding(&self) -> &Encoding {
        Encoding::from_ptr(unsafe { z_sample_encoding(self.zloan()) })
    }

    pub fn payload(&self) -> &ZBytes {
        ZBytes::from_ptr(unsafe { z_sample_payload(self.zloan()) })
    }

    pub fn keyexpr(&self) -> &KeyExpr {
        KeyExpr::from_ptr(unsafe { z_sample_keyexpr(self.zloan()) })
    }
}
