mod comm;
mod config;
mod messages;

use std::sync::Arc;

use yaair::yaair::{
    messages::{
        inbound::InboundMessage, outbound::OutboundMessage, serializer::Serializer,
        valuetree::ValueTree,
    },
    network::Network,
};
use zenoh_pico::keyexpr::KeyExpr;

pub use crate::{
    comm::ZenohNodeID,
    config::{
        NetworkConfig, ConfigBuilder,
        zenoh::{ZenohConfigBuilder},
    },
};
use crate::{
    comm::{
        CommunicationLayer, MessagePublisher, MessageSubscriber, MessageSubscriberOptions,
        TopicKeyExpr,
        zenoh::{Publisher, Session, Subscriber},
    },
    messages::store::AtomicMessagesStore,
};

pub struct NetworkContext<Ser: Sync + Send> {
    messages: AtomicMessagesStore<ZenohNodeID, ValueTree>,
    serializer: Ser,
}

pub struct ZenohNetwork<Ser: Sync + Send> {
    session: Session,
    context: Arc<NetworkContext<Ser>>,
    messages_publisher: Publisher,
    _messages_subscriber: Subscriber, // store it to keep it alive
}

impl<Ser: Serializer + Sync + Send + 'static> ZenohNetwork<Ser> {
    pub fn new(
        serializer: Ser,
        config: NetworkConfig,
    ) -> Result<Self, <Session as CommunicationLayer>::Err> {
        let session = Session::init(config.zenoh)?;
        let context = Arc::new(NetworkContext {
            messages: AtomicMessagesStore::new(config.lifespan),
            serializer,
        });

        let node_id = session.node_id();
        let base_keyexpr = KeyExpr::declare_topic(&config.base_keyexpr)?;
        let messages_keyexpr = base_keyexpr.declare_join("messages")?;
        let messages_publisher = Publisher::try_declare(
            &session,
            messages_keyexpr.declare_join(&node_id.to_string())?,
        )?;
        let messages_subscriber = Subscriber::try_declare_background(
            &session,
            messages_keyexpr.declare_join("*")?,
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

    fn on_outbound_message(
        outbound_message: OutboundMessage<ZenohNodeID>,
        context: &NetworkContext<Ser>,
    ) {
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

impl<Ser: Serializer + Sync + Send> Network<ZenohNodeID> for ZenohNetwork<Ser> {
    fn get_local_id(&self) -> ZenohNodeID {
        self.session.node_id()
    }

    fn prepare_outbound(&mut self, outbound_message: Vec<u8>) {
        log::debug!("Payload size: {}", outbound_message.len());
        match self.messages_publisher.put_message(outbound_message) {
            Ok(_) => log::info!("Message published successfully"),
            Err(e) => log::warn!("Error publishing message: {e}"),
        }
    }

    fn prepare_inbound(&mut self) -> InboundMessage<ZenohNodeID> {
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
