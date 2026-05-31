use std::{cmp::Ordering, collections::HashMap, thread::sleep, time::Duration};

use embassy_executor::Spawner;
use examples_common::{esp, zenoh};
use uuid::Uuid;
use yaair::yaair::{
    aggregate::{Aggregate, AggregateError, VM},
    data::field::Field,
    engine::Engine,
};
use yaair_zenoh_network::ZenohNetwork;
use yaair_serde::yaair_serde::json::JsonSerializer;
use zenoh_pico::{session::Session, zid::ZId};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Node {
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
    let network = ZenohNetwork::new(session, JsonSerializer, Default::default())
        .expect("Failed to create zenoh network");

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
    esp::init();

    let wifi = esp::start_wifi().await;
    let session = zenoh::start_session(wifi.netif().get_name(), Some(node.uuid()));
    spawner.spawn(gradient_task(node, session).expect("Failed to create gradient task"));
}
