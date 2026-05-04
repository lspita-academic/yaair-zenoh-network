use std::time::Duration;

use embassy_executor::Spawner;
use embassy_time::Timer;
use esp_idf_platform::wifi::{ConnectedWifi, Wifi, config::WifiConfig};
use esp_idf_svc::log::EspLogger;
use static_cell::StaticCell;
use zenoh_pico::{
    config::{ConfigBuilder, ConfigMode},
    session::Session,
    zbytes::TryIntoZBytes,
};

static ZENOH_SESSION: StaticCell<Session> = StaticCell::new();
static WIFI: StaticCell<ConnectedWifi<'static>> = StaticCell::new();

#[embassy_executor::task]
async fn pong(zenoh_session: &'static Session) {
    log::info!("Starting pong task");
    let publisher = zenoh_session
        .declare_publisher(
            &"pong/value".parse().expect("Pong keyexpr should be valid"),
            None,
        )
        .expect("Failed to declare pong publisher");
    let subscriber = zenoh_session
        .declare_subscriber_async(
            &"ping/value".parse().expect("Ping keyexpr should be valid"),
            None,
        )
        .expect("Failed to declare ping subscriber");

    loop {
        log::info!("Waiting ping");
        let sample = subscriber.recv_async().await;
        let bytes: Vec<_> = sample.payload().into_iter().collect();
        let ping: usize = postcard::from_bytes(&bytes).expect("Failed to deserialize ping");
        log::info!("Received ping: {ping}");
        Timer::after_secs(2).await;
        let pong = ping + 1;
        log::info!("Publishing pong: {pong}");
        let payload = postcard::to_allocvec(&pong)
            .expect("Failed to serialize pong")
            .try_into_zbytes()
            .expect("Failed to create pong payload");
        publisher
            .put(payload, None)
            .expect("Failed to publish pong");
        log::info!("Published pong");
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
    let zenoh_config = ConfigBuilder::default()
        .mode(ConfigMode::Peer)
        .scouting_timeout(Duration::from_secs(30))
        .multicast_locator(&format!("udp/224.0.0.224:7446#iface={if_name}"))
        .listen(&format!("udp/224.0.0.224:7447#iface={if_name}"))
        .build()
        .expect("Failed to build Zenoh config");

    let zenoh_session = ZENOH_SESSION
        .init(Session::open(zenoh_config, None).expect("Failed to open zenoh session"));
    spawner.spawn(pong(zenoh_session).expect("Failed to spawn pong task"));
}
