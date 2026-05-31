mod wifi;

use static_cell::StaticCell;
use wifi::{ConnectedWifi, Wifi, config::WifiConfig};

static WIFI: StaticCell<ConnectedWifi<'static>> = StaticCell::new();

pub fn init() {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::init_from_env();
}

pub async fn start_wifi() -> &'static mut ConnectedWifi<'static> {
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

    wifi
}
