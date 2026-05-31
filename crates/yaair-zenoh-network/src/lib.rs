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

use crate::{
    comm::{
        CommunicationLayer, MessagePublisher, MessageSubscriber, MessageSubscriberOptions,
        TopicKeyExpr,
        zenoh::{KeyExpr, Publisher, Session, Subscriber},
    },
    messages::store::AtomicMessagesStore,
};

pub struct NetworkContext<Ser> {
    messages: AtomicMessagesStore<ValueTree>,
    serializer: Ser,
}

pub struct NetworkConfig {
    pub base_keyexpr: KeyExpr,
    pub lifespan: Duration,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            base_keyexpr: KeyExpr::declare("yaair/network/zenoh")
                .expect("Failed to generate default base keyexpr for network"),
            lifespan: Duration::from_secs(10),
        }
    }
}

pub struct ZenohNetwork<'a, S> {
    session: &'a Session,
    context: Arc<NetworkContext<S>>,
    messages_publisher: Publisher,
    _messages_subscriber: Subscriber, // store it to keep it alive
}

impl<'a, S: Serializer> ZenohNetwork<'a, S> {
    pub fn new(
        session: &'a Session,
        serializer: S,
        config: NetworkConfig,
    ) -> Result<Self, <Session as CommunicationLayer>::Err> {
        let context = Arc::new(NetworkContext {
            messages: AtomicMessagesStore::new(config.lifespan),
            serializer,
        });

        let zid = session.zid();
        let messages_keyexpr = config.base_keyexpr.join(&KeyExpr::new("messages")?)?;
        let messages_publisher =
            Publisher::try_declare(session, &messages_keyexpr.declare_join(&zid.to_string())?)?;
        let messages_subscriber = Subscriber::try_declare(
            session,
            &messages_keyexpr.declare_join("*")?,
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
        outbound_message: OutboundMessage<<Session as CommunicationLayer>::Id>,
        context: &NetworkContext<S>,
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

impl<S: Serializer> Network<<Session as CommunicationLayer>::Id> for ZenohNetwork<'_, S> {
    fn get_local_id(&self) -> <Session as CommunicationLayer>::Id {
        self.session.zid()
    }

    fn prepare_outbound(&mut self, outbound_message: Vec<u8>) {
        let keyexpr = self.messages_publisher.publishing_keyexpr();
        log::info!("Publishing message to {keyexpr}");
        log::debug!("Payload size: {}", outbound_message.len());
        match self.messages_publisher.put_message(outbound_message) {
            Ok(_) => log::info!("Message published successfully"),
            Err(e) => log::warn!("Error publishing message: {e}"),
        }
    }

    fn prepare_inbound(&mut self) -> InboundMessage<<Session as CommunicationLayer>::Id> {
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
