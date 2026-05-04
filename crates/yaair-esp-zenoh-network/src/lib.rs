use std::{ffi::c_void, sync::Arc};

use yaair::yaair::{
    messages::{inbound::InboundMessage, serializer::Serializer},
    network::Network,
};
use zenoh_pico::{query::Query, session::{Session, queryreply::{Querier, Queryable}}, zbytes::ZBytes, zid::ZId, zvalue::ZValue};

pub struct ZenohPicoNetwork {
    payload: Arc<ZBytes>,
    queryable: Queryable,
    querier: Querier,
}

impl ZenohPicoNetwork {
    unsafe extern "C" fn handle_query(query: *const <Query as ZValue>::Value, context: *mut c_void) {

    }

    pub fn new(session: &Session) {

    }
}

impl<S: Serializer> Network<ZId, S> for ZenohPicoNetwork {
    fn prepare_outbound(&mut self, outbound_message: Vec<u8>) {
        todo!()
    }

    fn prepare_inbound(&mut self) -> InboundMessage<ZId> {
        todo!()
    }
}
