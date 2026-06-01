use std::{array, cmp::Ordering, collections::HashMap, thread::sleep, time::Duration};

use embassy_executor::Spawner;
use examples_common;
use yaair::yaair::{
    aggregate::{Aggregate, AggregateError, VM},
    data::field::Field,
    engine::Engine, network::Network,
};
use yaair_serde::yaair_serde::json::JsonSerializer;
use yaair_zenoh_network::{ZenohConfig, ZenohNetwork, ZenohNodeID};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Node {
    Node1,
    Node2,
    Node3,
}

impl Node {
    fn node_id(&self) -> ZenohNodeID {
        let value = match self {
            // full sequence cannot start with 0
            Self::Node1 => 0x11,
            Self::Node2 => 0x22,
            Self::Node3 => 0x33,
        };
        array::repeat(value).into()
    }

    fn is_source(&self) -> bool {
        *self == Self::Node1
    }
}

struct GradientEnv {
    node: Node,
    is_source: bool,
}

impl GradientEnv {
    fn distances(&self) -> Field<ZenohNodeID, f32> {
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
    vm: &mut VM<ZenohNodeID, JsonSerializer>,
) -> Result<f32, AggregateError> {
    let initial = f32::MAX;
    vm.share(&initial, |_, field| {
        let distances = field.aligned_map(&env.distances(), |a, b| a + b);
        let min_distance =
            *distances.min_by(|a, b| PartialOrd::partial_cmp(a, b).unwrap_or(Ordering::Greater));
        if env.is_source { 0.0 } else { min_distance }
    })
}

#[allow(clippy::print_stdout, clippy::print_stderr, clippy::use_debug)]
#[embassy_executor::task]
async fn gradient_task(node: Node, zenoh_config: ZenohConfig) {
    let network = ZenohNetwork::new(JsonSerializer, Default::default(), zenoh_config)
        .expect("Failed to create zenoh network");
    log::info!("Network id: {}", network.get_local_id());

    let env = GradientEnv {
        node,
        is_source: node.is_source(),
    };
    let mut engine = Engine::new(network, env, JsonSerializer, gradient);
    loop {
        match engine.cycle() {
            Ok(result) => log::info!("Gradient result: {result:?}"),
            Err(e) => log::warn!("Error during cycle: {e:?}"),
        }
        sleep(Duration::from_secs(1));
    }
}

pub async fn gradient_main(node: Node, spawner: Spawner) {
    examples_common::init();

    #[cfg(target_os = "espidf")]
    let interface = {
        let wifi = examples_common::esp::start_wifi().await;
        let if_name = wifi.netif().get_name().to_string();
        Some(if_name)
    };
    #[cfg(not(target_os = "espidf"))]
    let interface = None;

    let node_id = node.node_id();
    log::info!("Node id: {node_id}");
    let zenoh_config = ZenohConfig {
        interface,
        id: Some(node_id),
        ..Default::default()
    };
    spawner.spawn(gradient_task(node, zenoh_config).expect("Failed to create gradient task"));
}
