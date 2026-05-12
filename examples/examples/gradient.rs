use std::{cmp::Ordering, collections::HashMap, thread::sleep, time::Duration};

use embassy_executor::Spawner;
use examples::{esp, zenoh};
use uuid::Uuid;
use yaair::yaair::{
    aggregate::{Aggregate, AggregateError, VM},
    data::field::Field,
    engine::Engine,
};
use yaair_esp_zenoh_network::ZenohPicoNetwork;
use yaair_serde::yaair_serde::json::JsonSerializer;
use zenoh_pico::{session::Session, zid::ZId};

#[derive(Clone, Copy, PartialEq, Eq)]
enum Node {
    Node1 = 1,
    Node2 = 2,
    Node3 = 3,
}

impl Node {
    fn uuid(&self) -> Uuid {
        Uuid::from_bytes(core::array::repeat(*self as u8))
    }

    fn zid(&self) -> ZId {
        self.uuid().into()
    }

    fn is_source(&self) -> bool {
        *self == Self::Node1
    }
}

const fn node() -> Option<Node> {
    #[cfg(feature = "node1")]
    return Some(Node::Node1);

    #[cfg(feature = "node2")]
    return Some(Node::Node2);

    #[cfg(feature = "node3")]
    return Some(Node::Node3);

    #[allow(unreachable_code)] // instead of not all features
    None
}

struct GradientEnv {
    node: Node,
    is_source: bool,
}

impl GradientEnv {
    fn distances(&self) -> Field<ZId, f32> {
        match self.node {
            Node::Node1 => Field::new(0.0, HashMap::from([(Node::Node2.zid(), 1.0)])),
            Node::Node2 => Field::new(
                0.0,
                HashMap::from([(Node::Node1.zid(), 1.0), (Node::Node3.zid(), 1.5)]),
            ),
            Node::Node3 => Field::new(0.0, HashMap::from([(Node::Node2.zid(), 1.5)])),
        }
    }
}

fn gradient(env: &GradientEnv, vm: &mut VM<ZId, JsonSerializer>) -> Result<f32, AggregateError> {
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
async fn gradient_task(node: Node, session: &'static Session) {
    let network = ZenohPicoNetwork::new(session, JsonSerializer, Default::default())
        .expect("Failed to create zenoh network");

    let env = GradientEnv {
        node,
        is_source: node.is_source(),
    };
    let mut engine = Engine::new(session.zid(), network, env, JsonSerializer, gradient);
    for _ in 0..10 {
        match engine.cycle() {
            Ok(result) => log::info!("Gradient result: {result:?}"),
            Err(e) => log::warn!("Error during cycle: {e:?}"),
        }
        sleep(Duration::from_secs(1));
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    esp::init();

    let Some(node) = self::node() else {
        return;
    };

    let wifi = esp::start_wifi().await;
    let session = zenoh::start_session(wifi.netif().get_name(), Some(node.uuid()));
    spawner.spawn(gradient_task(node, session).expect("Failed to create gradient task"));
}
