use std::sync::Arc;

use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, signal::Signal};

use crate::{
    keyexpr::KeyExpr,
    result::ZenohResult,
    zoptions::ZOptions,
    zvalue::{ZClone, ZClosure},
};

pub trait KeyHandler
where
    Self: Sized,
{
    type Declarer;
    type Options: ZOptions;
    type Closure: ZClosure;

    fn from_declaration(
        declarer: &Self::Declarer,
        key: &KeyExpr,
        closure: Self::Closure,
        options: Option<Self::Options>,
    ) -> ZenohResult<Self>;
}

pub struct AsyncHandler<Handler, T> {
    handler: Handler,
    signal: Arc<Signal<CriticalSectionRawMutex, T>>,
}

impl<Handler, T> AsyncHandler<Handler, T>
where
    Handler: KeyHandler,
    T: ZClone<Value = <Handler::Closure as ZClosure>::CallbackValue>,
{
    pub fn new(handler: Handler, signal: Arc<Signal<CriticalSectionRawMutex, T>>) -> Self {
        Self { handler, signal }
    }

    pub fn handler(&self) -> &Handler {
        &self.handler
    }

    pub fn handler_mut(&mut self) -> &mut Handler {
        &mut self.handler
    }

    pub async fn recv_async(&self) -> T {
        self.signal.wait().await
    }
}
