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
    z_query_payload, z_query_reply, z_query_reply_del, z_query_reply_del_options_default,
    z_query_reply_del_options_t, z_query_reply_err, z_query_reply_err_options_default,
    z_query_reply_err_options_t, z_query_reply_options_default, z_query_reply_options_t,
    z_query_target_t_Z_QUERY_TARGET_ALL, z_query_target_t_Z_QUERY_TARGET_ALL_COMPLETE,
    z_query_target_t_Z_QUERY_TARGET_BEST_MATCHING, z_reply_err, z_reply_err_encoding,
    z_reply_err_payload, z_reply_is_ok, z_reply_keyexpr_t_Z_REPLY_KEYEXPR_ANY,
    z_reply_keyexpr_t_Z_REPLY_KEYEXPR_MATCHING_QUERY, z_reply_ok,
};

use crate::{
    keyexpr::KeyExpr,
    message::Encoding,
    result::{IntoZenohResult, ZenohResult},
    sample::Sample,
    zbytes::{IntoZBytes, ZBytes},
    zoptions::{ZOptionsInit, options_ptr},
    zstring::ZString,
    zvalue::{ZOwn, ZValue, ZView},
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

#[zwrap(base(name = "closure_query"), zvalue, zown, zclosure(callback_ty = <Query as ZValue>::Value))]
pub struct QueryClosure;

impl ZOptionsInit for z_query_reply_options_t {
    fn zinit(&mut self) {
        unsafe {
            z_query_reply_options_default(self);
        }
    }
}

impl ZOptionsInit for z_query_reply_del_options_t {
    fn zinit(&mut self) {
        unsafe {
            z_query_reply_del_options_default(self);
        }
    }
}

impl ZOptionsInit for z_query_reply_err_options_t {
    fn zinit(&mut self) {
        unsafe {
            z_query_reply_err_options_default(self);
        }
    }
}

impl Query {
    pub fn keyexpr(&self) -> &KeyExpr {
        KeyExpr::from_ptr(unsafe { z_query_keyexpr(self.zloan()) })
    }

    pub fn parameters(&self) -> &ZString {
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

    fn default_reply_keyexpr<'a>(&'a self, keyexpr: Option<&'a KeyExpr>) -> &'a KeyExpr {
        keyexpr.unwrap_or_else(|| self.keyexpr())
    }

    pub fn reply<Payload: IntoZBytes>(
        &self,
        payload: Payload,
        keyexpr: Option<&KeyExpr>,
        options: Option<z_query_reply_options_t>,
    ) -> ZenohResult<()> {
        let options = options_ptr(options.as_ref());
        let keyexpr = self.default_reply_keyexpr(keyexpr);
        unsafe {
            z_query_reply(
                self.zloan(),
                keyexpr.zloan(),
                &mut payload.into_zbytes().zmove(),
                options,
            )
            .into_zresult()
        }
    }

    pub fn reply_del<Payload: IntoZBytes>(
        &self,
        keyexpr: Option<&KeyExpr>,
        options: Option<z_query_reply_del_options_t>,
    ) -> ZenohResult<()> {
        let options = options_ptr(options.as_ref());
        let keyexpr = self.default_reply_keyexpr(keyexpr);
        unsafe { z_query_reply_del(self.zloan(), keyexpr.zloan(), options).into_zresult() }
    }

    pub fn reply_err<Payload: IntoZBytes>(
        &self,
        payload: Payload,
        options: Option<z_query_reply_err_options_t>,
    ) -> ZenohResult<()> {
        let options = options_ptr(options.as_ref());
        unsafe {
            z_query_reply_err(self.zloan(), &mut payload.into_zbytes().zmove(), options)
                .into_zresult()
        }
    }
}

#[zwrap(base(name = "reply"), zvalue, zown, zclone)]
pub struct Reply;

#[zwrap(base(name = "closure_reply"), zvalue, zown, zclosure(callback_ty = <Reply as ZValue>::Value))]
pub struct ReplyClosure;

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
