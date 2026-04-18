use std::time::Duration;

use embassy_executor::{Spawner, SpawnerTraceExt};
use embassy_time::Timer;
use esp_idf_svc::log::EspLogger;
use esp_idf_svc::wifi::{AsyncWifi, EspWifi};
use static_cell::StaticCell;
use ping_pong::wifi;
use zenoh_pico::{
    config::{ZenohConfigBuilder, ZenohConfigMode},
    session::ZenohSession,
};

static ZENOH_SESSION: StaticCell<ZenohSession> = StaticCell::new();
static WIFI: StaticCell<AsyncWifi<EspWifi<'static>>> = StaticCell::new();

#[embassy_executor::task]
async fn ping(zenoh_session: &'static ZenohSession) {
    log::info!("Starting ping task");
    let publisher = zenoh_session.publisher("ping/value");
    let subscriber = zenoh_session.subscriber("pong/value");

    Timer::after_secs(2).await;
    zenoh_session.print_peers_zid();
    let mut count = 0;
    loop {
        let ping = count.to_string();
        Timer::after_millis(2000).await;
        publisher.put(&ping);
        let pong = subscriber.recv_async().await;
        log::info!("Received pong: {}", pong);
        count += 1;
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    esp_idf_svc::sys::link_patches();
    EspLogger::initialize_default();

    let mut wifi = WIFI.init(wifi::get_wifi().expect("Unable to initialize wifi"));
    wifi::connect_wifi(&mut wifi)
        .await
        .unwrap_or_else(|err| panic!("Wifi connection raised error: {:?}", err));

    let net_if = wifi.wifi().sta_netif();
    let if_name = net_if.get_name();
    let ip_info = net_if.get_ip_info().expect("Error getting IP info");
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
        .spawn_named("ping", ping(zenoh_session))
        .expect("Failed to spawn ping task");
}
