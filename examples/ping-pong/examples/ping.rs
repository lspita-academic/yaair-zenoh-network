#[cfg(target_os = "espidf")]
#[embassy_executor::main]
async fn main(spawner: embassy_executor::Spawner) {
    examples_common::init();

    let wifi = examples_common::esp::start_wifi().await;
    let session = ping_pong_example::start_zenoh_session(wifi.netif().get_name());
    spawner.spawn(ping_pong_example::ping::ping(session).expect("Failed to create ping task"));
}

#[cfg(not(target_os = "espidf"))]
fn main() {
    panic!("This example is for esp-idf targets only");
}
