#[cfg(feature = "heartbeat")]
mod heartbeat;

use std::{array, cmp::Ordering, collections::HashMap, time::Duration};

use embassy_executor::Spawner;
use embassy_time::Timer;
use examples_common;
use yaair::yaair::{
    aggregate::{Aggregate, AggregateError, VM},
    data::field::Field,
    engine::Engine,
    network::Network,
};
use yaair_serde::yaair_serde::json::JsonSerializer;
#[cfg_attr(target_os = "espidf", allow(unused_imports))]
use yaair_zenoh_network::config::ConfigBuilderDefault;
use yaair_zenoh_network::{
    ZenohNetwork,
    config::{ConfigBuilder, ZenohConfigBuilder, ZenohNetworkConfig},
    id::ZenohNodeId,
};

pub type Serializer = JsonSerializer;
pub type EmbassyDuration = embassy_time::Duration;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Node {
    Node1,
    Node2,
    Node3,
}

impl Node {
    fn node_id(&self) -> ZenohNodeId {
        let value = match self {
            // full sequence cannot start with 0
            Self::Node1 => 0x11,
            Self::Node2 => 0x22,
            Self::Node3 => 0x33,
        };
        array::repeat(value).into()
    }

    #[cfg(feature = "heartbeat")]
    fn heartbeat_lifespan(&self) -> Option<Duration> {
        match self {
            Self::Node2 => Some(Duration::from_secs(3)),
            _ => None,
        }
    }
}

struct GradientEnv {
    node: Node,
}

impl GradientEnv {
    fn is_source(&self) -> bool {
        self.node == Node::Node1
    }

    fn distances(&self) -> Field<ZenohNodeId, f32> {
        match self.node {
            Node::Node1 => Field::new(0.0, HashMap::from([(Node::Node2.node_id(), 1.0)])),
            Node::Node2 => Field::new(
                0.0,
                HashMap::from([(Node::Node1.node_id(), 1.0), (Node::Node3.node_id(), 1.5)]),
            ),
            Node::Node3 => Field::new(0.0, HashMap::from([(Node::Node2.node_id(), 1.5)])),
        }
    }
}

fn gradient(
    env: &GradientEnv,
    vm: &mut VM<ZenohNodeId, Serializer>,
) -> Result<f32, AggregateError> {
    let initial = f32::MAX;
    vm.share(&initial, |_, field| {
        let distances = field.aligned_map(&env.distances(), |a, b| a + b);
        let min_distance =
            *distances.min_by(|a, b| PartialOrd::partial_cmp(a, b).unwrap_or(Ordering::Greater));
        if env.is_source() { 0.0 } else { min_distance }
    })
}

#[allow(clippy::print_stdout, clippy::print_stderr, clippy::use_debug)]
#[embassy_executor::task]
async fn gradient_task(node: Node, network: ZenohNetwork<Serializer>) {
    log::warn!("Gradient task started");

    let env = GradientEnv { node };
    let mut engine = Engine::new(network, env, Serializer {}, gradient);
    loop {
        match engine.cycle() {
            Ok(result) => log::info!("Gradient result: {result:?}"),
            Err(e) => log::warn!("Error during cycle: {e:?}"),
        }
        Timer::after(EmbassyDuration::from_secs(3)).await;
    }
}

pub async fn gradient_main(node: Node, spawner: Spawner) {
    examples_common::init();
    log::warn!("Heartbeat feature: {}", cfg!(feature = "heartbeat"));

    #[cfg(target_os = "espidf")]
    let zenoh_config_builder = {
        use yaair_zenoh_network::config::ZenohConfigBuilderInitOptions;

        let wifi = examples_common::esp::start_wifi().await;
        let interface = wifi.netif().get_name().to_string();
        ZenohConfigBuilder::new(ZenohConfigBuilderInitOptions {
            interface: interface.into(),
        })
        .set_default_options()
    };
    #[cfg(not(target_os = "espidf"))]
    let zenoh_config_builder = ZenohConfigBuilder::with_default_options();

    let node_id = node.node_id();
    log::info!("Node id: {node_id}");

    let zenoh_config = zenoh_config_builder
        .id(node_id)
        .build()
        .expect("Failed to create zenoh config");
    let network_config = ZenohNetworkConfig {
        lifespan: Duration::from_secs(15),
        ..zenoh_config.into()
    };
    let network =
        ZenohNetwork::new(Serializer {}, network_config).expect("Failed to create zenoh network");
    log::info!("Network id: {}", network.get_local_id());

    #[cfg(feature = "heartbeat")]
    {
        let heartbeat_publisher = network
            .declare_heartbeat_publisher()
            .expect("Failed to declare heartbeat publisher");

        spawner.spawn(
            heartbeat::periodic_heartbeat_task(
                heartbeat_publisher,
                EmbassyDuration::from_secs(10),
                node.heartbeat_lifespan(),
            )
            .expect("Failed to create heartbeat task"),
        );
    }
    spawner.spawn(gradient_task(node, network).expect("Failed to create gradient task"));
}
