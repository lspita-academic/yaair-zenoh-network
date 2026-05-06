use std::{cmp::Ordering, collections::HashMap, thread::sleep, time::Duration};

use embassy_executor::Spawner;
use examples::{esp, zenoh};
use yaair::yaair::{
    aggregate::{Aggregate, AggregateError, VM},
    data::field::Field,
    engine::Engine,
};
use yaair_esp_zenoh_network::ZenohPicoNetwork;
use yaair_serde::yaair_serde::json::JsonSerializer;
use zenoh_pico::{session::Session, zid::ZId};

struct GradientEnv {
    pub is_source: bool,
}

impl GradientEnv {
    fn distances(&self) -> Field<ZId, f32> {
        Field::new(
            0.0,
            HashMap::from([
                (ZId::new(std::array::repeat(1)), 1.0),
                (ZId::new(std::array::repeat(2)), 2.0),
                (ZId::new(std::array::repeat(3)), 1.5),
            ]),
        )
    }
}

fn gradient(env: &GradientEnv, vm: &mut VM<ZId, JsonSerializer>) -> Result<f32, AggregateError> {
    let initial = f32::MAX;
    vm.share(&initial, |_, field| {
        let distances = field.aligned_map(&env.distances(), |a, b| a + b);
        let min_distance =
            *distances.min_by(|a, b| PartialOrd::partial_cmp(&a, &b).unwrap_or(Ordering::Greater));
        if env.is_source { 0.0 } else { min_distance }
    })
}

#[allow(clippy::print_stdout, clippy::print_stderr, clippy::use_debug)]
#[embassy_executor::task]
async fn gradient_task(session: &'static Session) {
    let network = ZenohPicoNetwork::new(session, JsonSerializer, Default::default())
        .expect("Failed to create zenoh network");

    let env = GradientEnv { is_source: false };
    let mut engine = Engine::new(session.zid(), network, env, JsonSerializer, gradient);
    for _ in 0..10 {
        match engine.cycle() {
            Ok(result) => println!("Gradient result: {result:?}"),
            Err(e) => eprintln!("Error during cycle: {e:?}"),
        }
        sleep(Duration::from_secs(1));
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    esp::init();
    let wifi = esp::start_wifi().await;
    let session = zenoh::start_session(wifi.netif().get_name());
    spawner.spawn(gradient_task(session).expect("Failed to create gradient task"));
}
