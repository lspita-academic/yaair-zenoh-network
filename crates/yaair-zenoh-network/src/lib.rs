mod comm;
mod messages;

use std::{sync::Arc, time::Duration};

use yaair::yaair::{
    messages::{
        inbound::InboundMessage, outbound::OutboundMessage, serializer::Serializer,
        valuetree::ValueTree,
    },
    network::Network,
};
use zenoh_pico::{keyexpr::KeyExpr, result::ZenohResult, session::Session, zid::ZId};

use crate::{
    comm::pubsub::{MessagePublisher, MessageSubscriber, MessageSubscriberOptions},
    messages::store::AtomicMessagesStore,
};

pub struct NetworkContext<S> {
    messages: AtomicMessagesStore<ValueTree>,
    serializer: S,
}

pub struct NetworkConfig {
    pub base_keyexpr: KeyExpr,
    pub lifespan: Duration,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            base_keyexpr: KeyExpr::new("yaair/network/zenoh")
                .expect("Failed to generate default base keyexpr for network"),
            lifespan: Duration::from_secs(10),
        }
    }
}

pub struct ZenohPicoNetwork<'a, S> {
    session: &'a Session,
    context: Arc<NetworkContext<S>>,
    messages_publisher: MessagePublisher,
    _messages_subscriber: MessageSubscriber, // store it to keep it alive
}

impl<'a, S: Serializer> ZenohPicoNetwork<'a, S> {
    pub fn new(session: &'a Session, serializer: S, config: NetworkConfig) -> ZenohResult<Self> {
        let context = Arc::new(NetworkContext {
            messages: AtomicMessagesStore::new(config.lifespan),
            serializer,
        });

        let zid = session.zid();
        let messages_keyexpr = config
            .base_keyexpr
            .join_autocanonize(&KeyExpr::new("messages")?)?;
        let messages_publisher = MessagePublisher::new(
            session,
            &messages_keyexpr.join_autocanonize(&KeyExpr::new(&zid.to_string())?)?,
        )?;
        let messages_subscriber = MessageSubscriber::new(
            session,
            &messages_keyexpr,
            MessageSubscriberOptions {
                callback: Self::on_outbound_message,
                context: context.clone(),
            },
        )?;

        Ok(Self {
            session,
            context,
            messages_publisher,
            _messages_subscriber: messages_subscriber,
        })
    }

    fn on_outbound_message(outbound_message: OutboundMessage<ZId>, context: &NetworkContext<S>) {
        log::info!("Sender: {}", outbound_message.sender);
        match context
            .messages
            .store(outbound_message.sender, outbound_message.into_inner())
        {
            Ok(_) => log::info!("Message stored successfully"),
            Err(e) => log::warn!("Failed to store message: {e}"),
        }
    }
}

impl<S: Serializer> Network<ZId> for ZenohPicoNetwork<'_, S> {
    fn get_local_id(&self) -> ZId {
        self.session.zid()
    }

    fn prepare_outbound(&mut self, outbound_message: Vec<u8>) {
        let keyexpr = self.messages_publisher.publisher().keyexpr();
        log::info!("Publishing message to {keyexpr}");
        log::debug!("Payload size: {}", outbound_message.len());
        match self.messages_publisher.put(outbound_message) {
            Ok(_) => log::info!("Message published successfully"),
            Err(e) => log::warn!("Error publishing message: {e}"),
        }
    }

    fn prepare_inbound(&mut self) -> InboundMessage<ZId> {
        log::info!("Preparing inbound message");
        let messages = &self.context.messages;
        log::debug!("Preparing snapshot of messages");
        let snapshot = match messages.clear_dead().and_then(|_| messages.snapshot()) {
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
        let inbound_message_map = snapshot
            .into_iter()
            .map(|(zid, message)| (zid, message.into_inner()))
            .collect();
        let inbound_message = InboundMessage::new(inbound_message_map);
        log::info!("Inbound message created");
        inbound_message
    }
}
