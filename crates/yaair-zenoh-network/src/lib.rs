//! A zenoh based network for [YAAIR](yaair).
//!
//! It supports both standard and embedded targets by using
//! [Zenoh](https://github.com/eclipse-zenoh/zenoh) and
//! [Zenoh pico](https://github.com/eclipse-zenoh/zenoh-pico).
//!
//! **FOR EMBEDDED, ONLY ESP-IDF BASED TARGETS ARE SUPPORTED**
//!
//! # Examples
//!
//! You can find a full aggregate computing example using yaair
//! [here](https://github.com/lspita-academic/yaair-zenoh-network/tree/main/examples/gradient)
//!
//! ```
//! # use yaair_zenoh_network::{
//! #     ZenohNetwork,
//! #     id::ZenohNodeId,
//! #     config::{
//! #         ZenohNetworkConfig,
//! #         ZenohConfig,
//! #         ZenohConfigBuilder,
//! #         ConfigBuilder,
//! #         ConfigBuilderDefault,
//! #         ZenohConfigBuilderInitOptions
//! #     }
//! # };
//! # use yaair_serde::yaair_serde::json::JsonSerializer;
//! # use yaair::yaair::aggregate::{VM, AggregateError};
//! #
//! // Create the zenoh config
//! #[cfg(target_os = "espidf")]
//! let zenoh_config_builder = {
//!     // The zenoh pico implementation requires some options
//!     let interface = "lo0";
//!     ZenohConfigBuilder::new(ZenohConfigBuilderInitOptions {
//!         interface: interface.into(),
//!     })
//!     .set_default_options()
//! };
//! #[cfg(not(target_os = "espidf"))]
//! // With standard zenoh a default builder can be initialized
//! let zenoh_config_builder = ZenohConfigBuilder::with_default_options();
//!
//! let zenoh_config = zenoh_config_builder
//!     .build()
//!     .expect("Failed to build the zenoh config");
//!
//! // Create the network config
//! let network_config = zenoh_config.into();
//! // You can also override only specific options like this
//! // let network_config = ZenohNetworkConfig {
//! //      lifespan: Duration::from_secs(10),
//! //      ..zenoh_config.into(),
//! // };
//!
//! // Create the network
//! // NOTE: The network and engine MUST use the same serializer
//! let zenoh_network =
//!     ZenohNetwork::new(JsonSerializer, network_config).expect("Failed to create zenoh network");
//!
//! // You are now ready to use it in the engine!
//! struct DummyEngineEnv;
//!
//! fn engine_program(
//!     env: &DummyEngineEnv,
//!     vm: &mut VM<ZenohNodeId, JsonSerializer>,
//! ) -> Result<(), AggregateError> {
//!     Ok(())
//! }
//!
//! let engine = Engine::new(
//!     zenoh_network,
//!     DummyEngineEnv,
//!     JsonSerializer,
//!     engine_program,
//! );
//! ```
//!
//! # Features
//!
//! - `heartbit`: enables the ability to use keepalive messages to be able to
//!   notify other peers of the node existance. See the [`heartbit`] module.

pub(crate) mod comm;
pub mod config;
#[cfg(feature = "heartbit")]
pub mod heartbit;
pub mod id;
pub(crate) mod messages;

#[cfg(zenoh_impl = "zenoh_full")]
#[path = "zenoh_full/mod.rs"]
mod zenoh_impl;

#[cfg(zenoh_impl = "zenoh_pico")]
#[path = "zenoh_pico/mod.rs"]
mod zenoh_impl;

use std::sync::Arc;

use itertools::Itertools;
use yaair::yaair::{
    messages::{
        inbound::InboundMessage, outbound::OutboundMessage, serializer::Serializer,
        valuetree::ValueTree,
    },
    network::Network,
};

#[cfg(feature = "heartbit")]
use crate::heartbit::{Heartbit, HeartbitPublisher};
pub use crate::zenoh_impl::ZenohError;
use crate::{
    comm::{
        CommunicationLayer, MessagePublisher, MessageSubscriber, MessageSubscriberOptions,
        TopicKeyExpr,
    },
    config::ZenohNetworkConfig,
    id::ZenohNodeId,
    messages::store::AtomicMessagesStore,
    zenoh_impl::comm::{KeyExpr, Publisher, Session, Subscriber},
};

pub(crate) struct NetworkContext<Ser: Sync + Send> {
    messages: AtomicMessagesStore<ZenohNodeId, ValueTree>,
    serializer: Ser,
    node_id: ZenohNodeId,
}

/// A zenoh based implementation of the yaair [`Network`] trait.
///
/// The network operates under a [base
/// keyexpr](ZenohNetworkConfig::base_keyexpr) namespace to prevent conflicts
/// with other topics.
///
/// If the [`heartbit`](crate#features) feature is enabled, it also listens for
/// heartbit messages and provides the ability to declare an
/// [`HeartbitPublisher`] to notify the other peers.
pub struct ZenohNetwork<Ser: Sync + Send> {
    session: Session,
    context: Arc<NetworkContext<Ser>>,
    messages_publisher: Publisher,
    #[cfg(feature = "heartbit")]
    heartbit_keyexpr: KeyExpr,
    // store subscribers to keep them alive
    _messages_subscriber: Subscriber,
    #[cfg(feature = "heartbit")]
    _heartbit_subscriber: Subscriber,
}

impl<Ser: Serializer + Sync + Send + 'static> ZenohNetwork<Ser> {
    /// Creates a new [`ZenohNetwork`] instance with the given serializer and
    /// configuration.
    pub fn new(serializer: Ser, config: ZenohNetworkConfig) -> Result<Self, ZenohError> {
        let session = Session::init(config.zenoh)?;
        let context = Arc::new(NetworkContext {
            messages: AtomicMessagesStore::new(config.lifespan),
            serializer,
            node_id: session.node_id(),
        });

        let base_keyexpr = KeyExpr::declare_topic(&config.base_keyexpr)?;
        let (messages_publisher, _messages_subscriber) =
            Self::init_messages(&session, context.clone(), &base_keyexpr)?;
        #[cfg(feature = "heartbit")]
        let (heartbit_keyexpr, _heartbit_subscriber) =
            Self::init_heartbit(&session, context.clone(), &base_keyexpr)?;

        Ok(Self {
            session,
            context,
            messages_publisher,
            #[cfg(feature = "heartbit")]
            heartbit_keyexpr,
            _messages_subscriber,
            #[cfg(feature = "heartbit")]
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

    #[cfg(feature = "heartbit")]
    fn init_heartbit(
        session: &Session,
        context: Arc<NetworkContext<Ser>>,
        base_keyexpr: &KeyExpr,
    ) -> Result<(KeyExpr, Subscriber), ZenohError> {
        let heartbit_keyexpr = base_keyexpr.declare_join("heartbit")?;
        let subscriber = Subscriber::try_declare_background(
            &session,
            heartbit_keyexpr.star()?,
            MessageSubscriberOptions {
                context,
                callback: Self::on_heartbit,
            },
        )?;
        Ok((heartbit_keyexpr, subscriber))
    }

    fn log_store_result<T>(id: ZenohNodeId) -> impl FnOnce(Option<T>) {
        return move |result| {
            if result.is_none() {
                log::warn!("New entity registered: {id}")
            }
        };
    }

    fn on_outbound_message(
        outbound_message: OutboundMessage<ZenohNodeId>,
        context: &NetworkContext<Ser>,
    ) {
        let sender = outbound_message.sender;
        if sender == context.node_id {
            return;
        }

        log::info!("Outbound message from: {}", sender);
        match context
            .messages
            .store_message(sender, outbound_message.into_inner())
            .map(Self::log_store_result(sender))
        {
            Ok(_) => log::info!("Message stored successfully"),
            Err(e) => log::warn!("Failed to store message: {e}"),
        }
    }

    #[cfg(feature = "heartbit")]
    fn on_heartbit(heartbit: Heartbit, context: &NetworkContext<Ser>) {
        let sender = heartbit.sender;
        if sender == context.node_id {
            return;
        }

        log::info!("Heartbit message from: {}", sender);

        let store_result = if let Some(lifespan) = heartbit.lifespan {
            log::info!("Storing updated lifespan [ms]: {}", lifespan.as_millis());
            context
                .messages
                .store_lifespan(sender, lifespan)
                .map(Self::log_store_result(sender))
        } else {
            context
                .messages
                .keep_alive(sender)
                .map(Self::log_store_result(sender))
        };
        match store_result {
            Ok(_) => log::info!("Heartbit stored successfully"),
            Err(e) => log::warn!("Failed to store heartbit: {e}"),
        }
    }

    /// Declares an [`HeartbitPublisher`] to notify the other peers of the node
    /// existance.
    #[cfg(feature = "heartbit")]
    pub fn declare_heartbit_publisher(&self) -> Result<HeartbitPublisher<Ser>, ZenohError> {
        let node_id = self.get_local_id();
        let keyexpr = self.heartbit_keyexpr.declare_join(&node_id.to_string())?;
        HeartbitPublisher::try_declare(&self.session, keyexpr, self.context.clone())
    }
}

impl<Ser: Serializer + Sync + Send> Network<ZenohNodeId> for ZenohNetwork<Ser> {
    fn get_local_id(&self) -> ZenohNodeId {
        self.session.node_id()
    }

    fn prepare_outbound(&mut self, outbound_message: Vec<u8>) {
        log::debug!("Preparing outbound message");
        match self.messages_publisher.put_message(outbound_message) {
            Ok(_) => log::info!("Message published successfully"),
            Err(e) => log::warn!("Error publishing message: {e}"),
        }
    }

    fn prepare_inbound(&mut self) -> InboundMessage<ZenohNodeId> {
        log::info!("Preparing inbound message");
        let messages = &self.context.messages;
        log::debug!("Preparing snapshot of messages");
        let snapshot = match messages.clear_dead().and_then(|expired| {
            if !expired.is_empty() {
                let expired_str = expired.into_iter().map(|e| e.to_string()).join(", ");
                log::warn!("Expired nodes: {expired_str}");
            }
            messages.messages_snapshot()
        }) {
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
