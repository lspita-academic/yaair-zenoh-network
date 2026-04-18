use std::time::Duration;

use embassy_executor::{Spawner, SpawnerTraceExt};
use embassy_time::Timer;
use esp_idf_svc::log::EspLogger;
use platform::wifi::{Wifi, WifiConnection, config::WifiConfig};
use static_cell::StaticCell;
use zenoh_pico::{
    config::{ZenohConfigBuilder, ZenohConfigMode},
    session::ZenohSession,
};

static ZENOH_SESSION: StaticCell<ZenohSession> = StaticCell::new();
static WIFI: StaticCell<WifiConnection<'static>> = StaticCell::new();

#[embassy_executor::task]
async fn pong(zenoh_session: &'static ZenohSession) {
    log::info!("Starting pong task");
    let publisher = zenoh_session.publisher("pong/value");
    let subscriber = zenoh_session.subscriber("ping/value");

    Timer::after_secs(2).await;
    zenoh_session.print_peers_zid();
    let mut count = 0;
    loop {
        let pong = count.to_string();
        let ping = subscriber.recv_async().await;
        log::info!("Received ping: {}", ping);
        Timer::after_millis(2000).await;
        publisher.put(&pong);
        count += 1;
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    esp_idf_svc::sys::link_patches();
    EspLogger::initialize_default();

    let wifi_config =
        WifiConfig::try_from_comptime_env().expect("Unable to initialize wifi config");
    let wifi = WIFI.init(
        Wifi::new()
            .expect("Unable to initialize wifi")
            .connect_with_config(&wifi_config)
            .await
            .expect("Unable to connect to wifi"),
    );

    let netif = wifi.netif();
    let if_name = netif.get_name();
    let ip_info = netif.get_ip_info().expect("Unable to get IP info");
    log::info!("WiFi interface: {}", if_name);
    log::info!("IP address: {}", ip_info.ip);

    let zenoh_config = ZenohConfigBuilder::default()
        .mode(ZenohConfigMode::Peer)
        .scouting_timeout(Duration::from_secs(30))
        .multicast_locator(&format!("udp/224.0.0.224:7446#iface={}", if_name))
        .listen(&format!("udp/224.0.0.224:7447#iface={}", if_name))
        .build();

    log::info!("Zenoh config mode: {:?}", zenoh_config.mode());

    let zenoh_session = ZENOH_SESSION.init(ZenohSession::open(zenoh_config, None));
    spawner
        .spawn_named("pong", pong(zenoh_session))
        .expect("Failed to spawn pong task");
}
