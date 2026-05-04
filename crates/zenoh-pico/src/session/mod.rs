pub(self) mod handler;
pub mod info;
pub mod matching;
pub mod pubsub;
pub mod queryreply;

use std::sync::Arc;

use embassy_sync::signal::Signal;
use zenoh_pico_macros::zwrap;
use zenoh_pico_sys::{
    z_close, z_close_options_t, z_declare_publisher, z_declare_querier, z_info_peers_zid,
    z_info_zid, z_open, z_open_options_default, z_open_options_t, z_publisher_options_t,
    z_querier_options_t, z_queryable_options_t, z_session_is_closed, z_subscriber_options_t,
};

use crate::{
    config::Config,
    keyexpr::KeyExpr,
    query::{Query, QueryClosure},
    result::{IntoZenohResult, ZenohResult},
    sample::{Sample, SampleClosure},
    session::{
        handler::{AsyncHandler, KeyHandler},
        info::PeersInfo,
        pubsub::{Publisher, Subscriber},
        queryreply::{Querier, Queryable},
    },
    zid::{ZId, ZIdClosure},
    zoptions::{ZOptionsInit, options_ptr, options_ptr_mut},
    zvalue::{ZClosure, ZOwn, ZValue},
};

impl ZOptionsInit for z_open_options_t {
    fn zinit(&mut self) {
        unsafe {
            z_open_options_default(self);
        }
    }
}

impl ZOptionsInit for z_close_options_t {
    fn zinit(&mut self) {
        // dummy struct
    }
}

#[zwrap(base(name = "session", family = "rc"), zvalue, zown)]
pub struct Session;

impl Session {
    pub fn open(config: Config, options: Option<z_open_options_t>) -> ZenohResult<Self> {
        let options = options_ptr(options.as_ref());
        let mut session = Self::uninitialized();
        session.with_zowned_mut(|z| unsafe {
            z_open(z, &mut config.zmove(), options).into_zresult()
        })?;
        Ok(session)
    }

    pub fn close(mut self, options: Option<z_close_options_t>) {
        let session_closed = unsafe { z_session_is_closed(self.zloan()) };
        if session_closed {
            return;
        }
        let options = options_ptr(options.as_ref());
        unsafe {
            z_close(self.zloan_mut(), options);
        }
    }

    pub fn declare_publisher(
        &self,
        key: &KeyExpr,
        options: Option<z_publisher_options_t>,
    ) -> ZenohResult<Publisher> {
        let options = options_ptr(options.as_ref());
        let mut publisher = Publisher::uninitialized();
        publisher.with_zowned_mut(|z| unsafe {
            z_declare_publisher(self.zloan(), z, key.zloan(), options).into_zresult()
        })?;
        Ok(publisher)
    }

    pub fn declare_subscriber(
        &self,
        key: &KeyExpr,
        closure: SampleClosure,
        options: Option<z_subscriber_options_t>,
    ) -> ZenohResult<Subscriber> {
        Subscriber::from_declaration(self, key, closure, options)
    }

    pub fn declare_subscriber_async(
        &self,
        key: &KeyExpr,
        options: Option<z_subscriber_options_t>,
    ) -> ZenohResult<AsyncHandler<Subscriber, Sample>> {
        let signal = Arc::new(Signal::new());
        let closure = SampleClosure::from_signal(signal.clone())?;
        let subscriber = self.declare_subscriber(key, closure, options)?;
        Ok(AsyncHandler::new(subscriber, signal))
    }

    pub fn declare_queryable(
        &self,
        key: &KeyExpr,
        closure: QueryClosure,
        options: Option<z_queryable_options_t>,
    ) -> ZenohResult<Queryable> {
        Queryable::from_declaration(self, key, closure, options)
    }

    pub fn declare_queryable_async(
        &self,
        key: &KeyExpr,
        options: Option<z_queryable_options_t>,
    ) -> ZenohResult<AsyncHandler<Queryable, Query>> {
        let signal = Arc::new(Signal::new());
        let closure = QueryClosure::from_signal(signal.clone())?;
        let queryable = self.declare_queryable(key, closure, options)?;
        Ok(AsyncHandler::new(queryable, signal))
    }

    pub fn declare_querier(
        &self,
        key: &KeyExpr,
        mut options: Option<z_querier_options_t>,
    ) -> ZenohResult<Querier> {
        let options = options_ptr_mut(options.as_mut());
        let mut querier = Querier::uninitialized();
        querier.with_zowned_mut(|z| unsafe {
            z_declare_querier(self.zloan(), z, key.zloan(), options).into_zresult()
        })?;
        Ok(querier)
    }

    pub fn zid(&self) -> ZId {
        unsafe { z_info_zid(self.zloan()) }.into()
    }

    pub fn peers(&self) -> ZenohResult<PeersInfo> {
        let signal = Arc::new(Signal::new());
        let zid_closure = ZIdClosure::from_signal(signal.clone())?;

        unsafe { z_info_peers_zid(self.zloan(), &mut zid_closure.zmove()).into_zresult() }?;
        Ok(PeersInfo { signal })
    }
}
