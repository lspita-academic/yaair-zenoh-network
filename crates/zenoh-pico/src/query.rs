use std::ptr::NonNull;

use ffi_utils::pointer::NonNullExtensions;
use num_enum::{IntoPrimitive, TryFromPrimitive, UnsafeFromPrimitive};
use zenoh_pico_macros::zwrap;
use zenoh_pico_sys::{
    _z_value_t, z_consolidation_mode_t_Z_CONSOLIDATION_MODE_AUTO,
    z_consolidation_mode_t_Z_CONSOLIDATION_MODE_LATEST,
    z_consolidation_mode_t_Z_CONSOLIDATION_MODE_MONOTONIC,
    z_consolidation_mode_t_Z_CONSOLIDATION_MODE_NONE, z_query_accepts_replies, z_query_attachment,
    z_query_consolidation_t, z_query_encoding, z_query_keyexpr, z_query_parameters,
    z_query_payload, z_query_target_t_Z_QUERY_TARGET_ALL,
    z_query_target_t_Z_QUERY_TARGET_ALL_COMPLETE, z_query_target_t_Z_QUERY_TARGET_BEST_MATCHING,
    z_reply_err, z_reply_err_encoding, z_reply_err_payload, z_reply_is_ok,
    z_reply_keyexpr_t_Z_REPLY_KEYEXPR_ANY, z_reply_keyexpr_t_Z_REPLY_KEYEXPR_MATCHING_QUERY,
    z_reply_ok,
};

use crate::{
    keyexpr::KeyExpr,
    message::Encoding,
    sample::Sample,
    zbytes::ZBytes,
    zstring::ZString,
    zvalue::{ZValue, ZView},
};

#[derive(IntoPrimitive, TryFromPrimitive, UnsafeFromPrimitive, Default)]
#[repr(u32)]
pub enum QueryTarget {
    #[default]
    BestMatching = z_query_target_t_Z_QUERY_TARGET_BEST_MATCHING,
    All = z_query_target_t_Z_QUERY_TARGET_ALL,
    AllComplete = z_query_target_t_Z_QUERY_TARGET_ALL_COMPLETE,
}

#[derive(IntoPrimitive, TryFromPrimitive, UnsafeFromPrimitive, Default)]
#[repr(i32)]
pub enum ConsolidationMode {
    #[default]
    Auto = z_consolidation_mode_t_Z_CONSOLIDATION_MODE_AUTO,
    None = z_consolidation_mode_t_Z_CONSOLIDATION_MODE_NONE,
    Monotonic = z_consolidation_mode_t_Z_CONSOLIDATION_MODE_MONOTONIC,
    Latest = z_consolidation_mode_t_Z_CONSOLIDATION_MODE_LATEST,
}

impl From<ConsolidationMode> for z_query_consolidation_t {
    fn from(value: ConsolidationMode) -> Self {
        Self { mode: value.into() }
    }
}

impl From<z_query_consolidation_t> for ConsolidationMode {
    fn from(value: z_query_consolidation_t) -> Self {
        unsafe { Self::unchecked_transmute_from(value.mode) }
    }
}

#[derive(IntoPrimitive, TryFromPrimitive, UnsafeFromPrimitive, Default)]
#[repr(u32)]
pub enum ReplyKeyexpr {
    Any = z_reply_keyexpr_t_Z_REPLY_KEYEXPR_ANY,
    #[default]
    MatchingQuery = z_reply_keyexpr_t_Z_REPLY_KEYEXPR_MATCHING_QUERY,
}

#[zwrap(base(name = "query", family = "rc"), zvalue, zown, zclone)]
pub struct Query;

impl Query {
    pub fn keyexpr(&self) -> &KeyExpr {
        KeyExpr::from_ptr(unsafe { z_query_keyexpr(self.zloan()) })
    }

    pub fn params(&self) -> &ZString {
        let mut view = <ZString as ZView>::ViewValue::default();
        unsafe {
            z_query_parameters(self.zloan(), &mut view);
        }
        ZView::from_zview(view)
    }

    pub fn payload(&self) -> &ZBytes {
        ZBytes::from_ptr(unsafe { z_query_payload(self.zloan()) })
    }

    pub fn encoding(&self) -> &Encoding {
        Encoding::from_ptr(unsafe { z_query_encoding(self.zloan()) })
    }

    pub fn attachment(&self) -> Option<&ZBytes> {
        NonNull::from_ptr(unsafe { z_query_attachment(self.zloan()) })
            .map(|nn| ZBytes::from_ptr(nn.as_ptr()))
    }

    pub fn accepts_replies(&self) -> ReplyKeyexpr {
        unsafe { ReplyKeyexpr::unchecked_transmute_from(z_query_accepts_replies(self.zloan())) }
    }
}

#[zwrap(base(name = "reply"), zvalue, zown, zclone)]
pub struct Reply;

#[zwrap(base(name = "reply_err"), zvalue(value_ty = _z_value_t), zown, zclone)]
pub struct ReplyError;

impl Reply {
    pub fn is_ok(&self) -> bool {
        unsafe { z_reply_is_ok(self.zloan()) }
    }

    pub fn is_err(&self) -> bool {
        !self.is_ok()
    }

    pub fn result(&self) -> Result<&Sample, &ReplyError> {
        if self.is_ok() {
            Ok(Sample::from_ptr(unsafe { z_reply_ok(self.zloan()) }))
        } else {
            Err(ReplyError::from_ptr(unsafe { z_reply_err(self.zloan()) }))
        }
    }
}

impl ReplyError {
    pub fn payload(&self) -> &ZBytes {
        ZBytes::from_ptr(unsafe { z_reply_err_payload(self.zloan()) })
    }

    pub fn encoding(&self) -> &Encoding {
        Encoding::from_ptr(unsafe { z_reply_err_encoding(self.zloan()) })
    }
}
