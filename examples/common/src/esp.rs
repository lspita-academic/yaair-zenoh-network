use esp_idf_platform::{
    self,
    wifi::{ConnectedWifi, Wifi, config::WifiConfig},
};
use static_cell::StaticCell;

static WIFI: StaticCell<ConnectedWifi<'static>> = StaticCell::new();

pub fn init() {
    esp_idf_platform::init(Default::default());
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
