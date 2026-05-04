use std::sync::Arc;

use embassy_sync::signal::Signal;
use zenoh_pico_macros::zwrap;
use zenoh_pico_sys::{
    z_declare_queryable, z_querier_get_matching_status, z_querier_get_options_default,
    z_querier_get_options_t, z_querier_get_with_parameters_substr, z_querier_keyexpr,
    z_querier_options_default, z_querier_options_t, z_queryable_keyexpr,
    z_queryable_options_default, z_queryable_options_t, z_undeclare_querier, z_undeclare_queryable,
};

use crate::{
    keyexpr::KeyExpr,
    query::{QueryClosure, Reply, ReplyClosure},
    result::{IntoZenohResult, ZenohResult},
    session::{
        Session,
        handler::{AsyncHandler, KeyHandler},
        matching::MatchingStatus,
    },
    zoptions::{ZOptions, ZOptionsInit, options_ptr, options_ptr_mut},
    zvalue::{ZClosure, ZOwn, ZValue},
};

#[zwrap(base(name = "queryable"), zvalue, zown(drop_zfn = z_undeclare_queryable))]
pub struct Queryable;

impl ZOptionsInit for z_queryable_options_t {
    fn zinit(&mut self) {
        unsafe {
            z_queryable_options_default(self);
        }
    }
}

impl KeyHandler for Queryable {
    type Declarer = Session;
    type Options = z_queryable_options_t;
    type Closure = QueryClosure;

    fn from_declaration(
        declarer: &Self::Declarer,
        key: &KeyExpr,
        closure: Self::Closure,
        options: Option<Self::Options>,
    ) -> ZenohResult<Self> {
        let options = options_ptr(options.as_ref());

        let mut queryable = Queryable::uninitialized();
        queryable.with_zowned_mut(|z| unsafe {
            z_declare_queryable(
                declarer.zloan(),
                z,
                key.zloan(),
                &mut closure.zmove(),
                options,
            )
            .into_zresult()
        })?;
        Ok(queryable)
    }
}

impl Queryable {
    pub fn keyexpr(&self) -> &KeyExpr {
        KeyExpr::from_ptr(unsafe { z_queryable_keyexpr(self.zloan()) })
    }
}

#[zwrap(base(name = "querier"), zvalue, zown(drop_zfn = z_undeclare_querier))]
pub struct Querier;

impl ZOptionsInit for z_querier_options_t {
    fn zinit(&mut self) {
        unsafe {
            z_querier_options_default(self);
        }
    }
}

impl Querier {
    pub fn matching_status(&self) -> ZenohResult<MatchingStatus> {
        let mut matching_status = MatchingStatus::uninitialized();
        unsafe {
            z_querier_get_matching_status(self.zloan(), matching_status.zloan_mut()).into_zresult()
        }?;
        Ok(matching_status)
    }

    pub fn keyexpr(&self) -> &KeyExpr {
        KeyExpr::from_ptr(unsafe { z_querier_keyexpr(self.zloan()) })
    }

    pub fn get(
        &self,
        handler: ReplyClosure,
        params: Option<&str>,
        options: Option<z_querier_get_options_t>,
    ) -> ZenohResult<QuerierGetHandle> {
        QuerierGetHandle::from_declaration(
            self,
            self.keyexpr(),
            handler,
            Some(QuerierGetHandleOptions {
                params: params.map(|s| s.to_owned()),
                get_options: options,
            }),
        )
    }

    pub fn get_async(
        &self,
        params: Option<&str>,
        options: Option<z_querier_get_options_t>,
    ) -> ZenohResult<AsyncHandler<QuerierGetHandle, Reply>> {
        let signal = Arc::new(Signal::new());
        let closure = ReplyClosure::from_signal(signal.clone())?;
        let get_handle = self.get(closure, params, options)?;
        Ok(AsyncHandler::new(get_handle, signal))
    }
}

pub struct QuerierGetHandle;

pub struct QuerierGetHandleOptions {
    params: Option<String>,
    get_options: Option<z_querier_get_options_t>,
}

impl ZOptionsInit for z_querier_get_options_t {
    fn zinit(&mut self) {
        unsafe {
            z_querier_get_options_default(self);
        }
    }
}

impl ZOptions for QuerierGetHandleOptions {
    fn zdefault() -> Self {
        Self {
            params: Default::default(),
            get_options: Default::default(),
        }
    }
}

impl KeyHandler for QuerierGetHandle {
    type Declarer = Querier;
    type Options = QuerierGetHandleOptions;
    type Closure = ReplyClosure;

    fn from_declaration(
        declarer: &Self::Declarer,
        _key: &KeyExpr,
        closure: Self::Closure,
        options: Option<Self::Options>,
    ) -> ZenohResult<Self> {
        let (params, mut get_options) = options
            .map(|o| (o.params, o.get_options))
            .unwrap_or_default();
        let params = params.unwrap_or_default();
        let get_options = options_ptr_mut(get_options.as_mut());

        unsafe {
            z_querier_get_with_parameters_substr(
                declarer.zloan(),
                params.as_ptr(),
                params.len(),
                &mut closure.zmove(),
                get_options,
            )
            .into_zresult()
        }?;
        Ok(Self)
    }
}
