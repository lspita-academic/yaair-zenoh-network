use std::time::Duration;

use embassy_executor::Spawner;
use embassy_time::Timer;
use esp_idf_platform::wifi::{Wifi, WifiConnection, config::WifiConfig};
use esp_idf_svc::log::EspLogger;
use static_cell::StaticCell;
use zenoh_pico::{
    config::{ZenohConfigBuilder, ZenohConfigMode},
    locator::{Locator, LocatorProtocol},
    session::ZenohSession,
};

static ZENOH_SESSION: StaticCell<ZenohSession> = StaticCell::new();
static WIFI: StaticCell<WifiConnection<'static>> = StaticCell::new();

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
        .mode(&ZenohConfigMode::Peer)
        .scouting_timeout(&Duration::from_secs(30))
        .multicast_locator(&Locator {
            protocol: LocatorProtocol::UDP,
            endpoint: "224.0.0.224:7446".parse().unwrap(),
            iface: Some(if_name.to_string()),
        })
        .listen(&Locator {
            protocol: LocatorProtocol::UDP,
            endpoint: "224.0.0.224:7447".parse().unwrap(),
            iface: Some(if_name.to_string()),
        })
        .build();

    let zenoh_session = ZENOH_SESSION.init(ZenohSession::open(zenoh_config, None));
    spawner.spawn(ping(zenoh_session).expect("Failed to spawn ping task"));
}
