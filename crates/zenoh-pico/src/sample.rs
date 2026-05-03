use std::ptr::NonNull;

use ffi_utils::pointer::NonNullExtensions;
use num_enum::{IntoPrimitive, TryFromPrimitive, UnsafeFromPrimitive};
use zenoh_pico_macros::zwrap;

use crate::{
    encoding::Encoding,
    keyexpr::KeyExpr,
    session::publisher::{CongestionControl, MessagePriority},
    sys::{
        z_sample_attachment, z_sample_congestion_control, z_sample_encoding, z_sample_express,
        z_sample_keyexpr, z_sample_kind, z_sample_kind_t_Z_SAMPLE_KIND_DELETE,
        z_sample_kind_t_Z_SAMPLE_KIND_PUT, z_sample_payload, z_sample_priority, z_sample_timestamp,
    },
    timestamp::Timestamp,
    zbytes::ZBytes,
    zvalue::ZValue,
};

#[zwrap(base(name = "sample"), zvalue, zown, zclone)]
pub struct Sample;

#[zwrap(base(name = "closure_sample"), zvalue, zown, zclosure)]
pub struct SampleClosure;

#[derive(IntoPrimitive, TryFromPrimitive, UnsafeFromPrimitive, Default)]
#[repr(u32)]
pub enum SampleKind {
    #[default]
    PUT = z_sample_kind_t_Z_SAMPLE_KIND_PUT,
    DELETE = z_sample_kind_t_Z_SAMPLE_KIND_DELETE,
}

impl Sample {
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

    pub fn priority(&self) -> MessagePriority {
        unsafe { MessagePriority::unchecked_transmute_from(z_sample_priority(self.zloan())) }
    }

    pub fn congestion_control(&self) -> CongestionControl {
        unsafe {
            CongestionControl::unchecked_transmute_from(z_sample_congestion_control(self.zloan()))
        }
    }

    pub fn is_express(&self) -> bool {
        unsafe { z_sample_express(self.zloan()) }
    }

    pub fn kind(&self) -> SampleKind {
        unsafe { SampleKind::unchecked_transmute_from(z_sample_kind(self.zloan())) }
    }
}
