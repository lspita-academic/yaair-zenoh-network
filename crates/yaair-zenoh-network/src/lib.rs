pub(crate) mod comm;
pub mod config;
pub(crate) mod messages;

#[cfg(zenoh_impl = "zenoh_full")]
#[path = "zenoh_full/mod.rs"]
mod zenoh_impl;

#[cfg(zenoh_impl = "zenoh_pico")]
#[path = "zenoh_pico/mod.rs"]
mod zenoh_impl;

use std::sync::Arc;

use yaair::yaair::{
    messages::{
        inbound::InboundMessage, outbound::OutboundMessage, serializer::Serializer,
        valuetree::ValueTree,
    },
    network::Network,
};

pub use crate::{comm::ZenohNodeId, zenoh_impl::ZenohError};
use crate::{
    comm::{
        CommunicationLayer, MessagePublisher, MessageSubscriber, MessageSubscriberOptions,
        TopicKeyExpr,
    },
    config::NetworkConfig,
    messages::{heartbit::Heartbit, store::AtomicMessagesStore},
    zenoh_impl::comm::{KeyExpr, Publisher, Session, Subscriber},
};

pub struct NetworkContext<Ser: Sync + Send> {
    messages: AtomicMessagesStore<ZenohNodeId, ValueTree>,
    serializer: Ser,
}

pub struct ZenohNetwork<Ser: Sync + Send> {
    session: Session,
    context: Arc<NetworkContext<Ser>>,
    messages_publisher: Publisher,
    // store subscribers to keep them alive
    _messages_subscriber: Subscriber,
    _heartbit_subscriber: Subscriber,
}

impl<Ser: Serializer + Sync + Send + 'static> ZenohNetwork<Ser> {
    pub fn new(serializer: Ser, config: NetworkConfig) -> Result<Self, ZenohError> {
        let session = Session::init(config.zenoh)?;
        let context = Arc::new(NetworkContext {
            messages: AtomicMessagesStore::new(config.lifespan),
            serializer,
        });

        let base_keyexpr = KeyExpr::declare_topic(&config.base_keyexpr)?;
        let (messages_publisher, _messages_subscriber) =
            Self::init_messages(&session, context.clone(), &base_keyexpr)?;
        let _heartbit_subscriber = Self::init_heartbit(&session, context.clone(), &base_keyexpr)?;

        Ok(Self {
            session,
            context,
            messages_publisher,
            _messages_subscriber,
            _heartbit_subscriber,
        })
    }

    fn init_messages(
        session: &Session,
        context: Arc<NetworkContext<Ser>>,
        base_keyexpr: &KeyExpr,
    ) -> Result<(Publisher, Subscriber), ZenohError> {
        let node_id = session.node_id();
        let messages_keyexpr = base_keyexpr.declare_join("messages")?;
        let messages_publisher = Publisher::try_declare(
            session,
            messages_keyexpr.declare_join(&node_id.to_string())?,
        )?;
        let messages_subscriber = Subscriber::try_declare_background(
            &session,
            messages_keyexpr.star()?,
            MessageSubscriberOptions {
                context,
                callback: Self::on_outbound_message,
            },
        )?;
        Ok((messages_publisher, messages_subscriber))
    }

    fn init_heartbit(
        session: &Session,
        context: Arc<NetworkContext<Ser>>,
        base_keyexpr: &KeyExpr,
    ) -> Result<Subscriber, ZenohError> {
        let heartbit_keyexpr = base_keyexpr.declare_join("heartbit")?;
        Subscriber::try_declare_background(
            &session,
            heartbit_keyexpr.star()?,
            MessageSubscriberOptions {
                context,
                callback: Self::on_heartbit,
            },
        )
    }

    fn on_outbound_message(
        outbound_message: OutboundMessage<ZenohNodeId>,
        context: &NetworkContext<Ser>,
    ) {
        log::info!("Outbound message from: {}", outbound_message.sender);
        match context
            .messages
            .store_message(outbound_message.sender, outbound_message.into_inner())
        {
            Ok(_) => log::info!("Message stored successfully"),
            Err(e) => log::warn!("Failed to store message: {e}"),
        }
    }

    fn on_heartbit(heartbit: Heartbit<ZenohNodeId>, context: &NetworkContext<Ser>) {
        log::info!("Heartbit message from: {}", heartbit.sender);
        let store_result = if let Some(lifespan) = heartbit.lifespan {
            log::info!("Storing updated lifespan [ms]: {}", lifespan.as_millis());
            context.messages.store_lifespan(heartbit.sender, lifespan)
        } else {
            context.messages.keep_alive(heartbit.sender)
        };
        match store_result {
            Ok(_) => log::info!("Heartbit stored successfully"),
            Err(e) => log::warn!("Failed to store heartbit: {e}"),
        }
    }
}

impl<Ser: Serializer + Sync + Send> Network<ZenohNodeId> for ZenohNetwork<Ser> {
    fn get_local_id(&self) -> ZenohNodeId {
        self.session.node_id()
    }

    fn prepare_outbound(&mut self, outbound_message: Vec<u8>) {
        log::debug!("Payload size: {}", outbound_message.len());
        match self.messages_publisher.put_message(outbound_message) {
            Ok(_) => log::info!("Message published successfully"),
            Err(e) => log::warn!("Error publishing message: {e}"),
        }
    }

    fn prepare_inbound(&mut self) -> InboundMessage<ZenohNodeId> {
        log::info!("Preparing inbound message");
        let messages = &self.context.messages;
        log::debug!("Preparing snapshot of messages");
        let snapshot = match messages
            .clear_dead()
            .and_then(|_| messages.messages_snapshot())
        {
            Ok(s) => {
                log::debug!("Snapshot created successfully");
                s
            }
            Err(e) => {
                log::warn!("Error creating messages snapshot: {e}");
                return Default::default();
            }
        };
        log::debug!("Creating inbound message");
        let inbound_message = InboundMessage::new(snapshot);
        log::info!("Inbound message created");
        inbound_message
    }
}
