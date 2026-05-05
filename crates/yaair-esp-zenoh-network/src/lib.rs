pub mod atomic;
pub mod message;

use std::{collections::HashMap, marker::PhantomData, sync::Arc, time::Duration};

use yaair::yaair::{
    messages::{inbound::InboundMessage, path::Path, serializer::Serializer, valuetree::ValueTree},
    network::Network,
};
use zenoh_pico::{keyexpr::KeyExpr, result::ZenohResult, session::Session, zid::ZId};

use crate::message::{
    Message,
    pubsub::{MessagePublisher, MessageSubscriber},
    store::AtomicMessagesStore,
};

pub struct NetworkContext<S> {
    messages: AtomicMessagesStore,
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
    context: Arc<NetworkContext<S>>,
    messages_publisher: MessagePublisher,
    _messages_subscriber: MessageSubscriber, // store it to keep it alive
    // the session should outlive the network to prevent being closed prematurely
    _phantom: PhantomData<&'a Session>,
}

impl<'a, S: Serializer> ZenohPicoNetwork<'a, S> {
    pub fn new(session: &'a Session, serializer: S, config: NetworkConfig) -> ZenohResult<Self> {
        let context = Arc::new(NetworkContext {
            messages: AtomicMessagesStore::new(config.lifespan),
            serializer,
        });

        let messages_keyexpr = config
            .base_keyexpr
            .join_autocanonize(&KeyExpr::new("messages")?)?;
        let messages_publisher = MessagePublisher::new(session, &messages_keyexpr)?;
        let messages_subscriber =
            MessageSubscriber::new(session, &messages_keyexpr, context.clone())?;

        Ok(Self {
            context,
            messages_publisher,
            _messages_subscriber: messages_subscriber,
            _phantom: PhantomData,
        })
    }
}

impl<S: Serializer> Network<ZId, S> for ZenohPicoNetwork<'_, S> {
    fn prepare_outbound(&mut self, outbound_message: Vec<u8>) {
        let message = Message::new(outbound_message);
        if let Err(e) = self
            .messages_publisher
            .put(message, &self.context.serializer)
        {
            let zid = self.messages_publisher.zid();
            log::warn!("Error sending message from {zid}: {e}")
        }
    }

    fn prepare_inbound(&mut self) -> InboundMessage<ZId> {
        let messages = &self.context.messages;
        let snapshot = match messages.clear_dead().and_then(|_| messages.snapshot()) {
            Ok(s) => s,
            Err(e) => {
                log::warn!("Error creating messages snapshot: {e}");
                return Default::default();
            }
        };
        let inbound_message_map = snapshot
            .into_iter()
            .map(|(key, value)| {
                let message: Message = value.into();
                (
                    key,
                    ValueTree::new(HashMap::from([(
                        // TODO: ask what this path is
                        Path::new(Vec::<String>::default()),
                        message.into(),
                    )])),
                )
            })
            .collect();
        InboundMessage::new(inbound_message_map)
    }
}
