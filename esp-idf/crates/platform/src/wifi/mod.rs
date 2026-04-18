// https://github.com/esp-rs/esp-idf-svc/blob/master/examples/wifi_async.rs
use esp_idf_svc::{
    eventloop::EspSystemEventLoop,
    hal::peripherals::Peripherals,
    netif::EspNetif,
    nvs::EspDefaultNvsPartition,
    sys::EspError,
    timer::EspTaskTimerService,
    wifi::{AsyncWifi, EspWifi, WifiDriver},
};

pub mod config;

use config::WifiConfig;

type WifiImpl<'a> = AsyncWifi<EspWifi<'a>>;

pub struct Wifi<'a>(WifiImpl<'a>);

impl<'a> From<WifiImpl<'a>> for Wifi<'a> {
    fn from(value: WifiImpl<'a>) -> Self {
        Self(value)
    }
}

impl<'a> Wifi<'a> {
    pub fn new() -> Result<Self, EspError> {
        let peripherals = Peripherals::take()?;
        let sysloop = EspSystemEventLoop::take()?;
        let nvs = EspDefaultNvsPartition::take().ok();
        let timer_service = EspTaskTimerService::new()?;

        let driver = WifiDriver::new(peripherals.modem, sysloop.clone(), nvs)?;
        let esp_wifi = EspWifi::wrap(driver)?;
        Ok(WifiImpl::wrap(esp_wifi, sysloop, timer_service)?.into())
    }

    pub async fn connect_with_config(
        mut self,
        config: &WifiConfig,
    ) -> Result<WifiConnection<'a>, EspError> {
        let wifi = &mut self.0;
        wifi.set_configuration(config.esp_value())?;
        wifi.start().await?;
        log::info!("Wifi started");

        wifi.connect().await?;
        log::info!("Wifi connected");

        wifi.wait_netif_up().await?;
        log::info!("Wifi netif up");
        Ok(WifiConnection(self))
    }

    fn esp_value(&self) -> &WifiImpl<'a> {
        &self.0
    }

    fn esp_value_mut(&mut self) -> &mut WifiImpl<'a> {
        &mut self.0
    }
}

pub struct WifiConnection<'a>(Wifi<'a>);

impl<'a> WifiConnection<'a> {
    pub async fn disconnect(mut self) -> Result<Wifi<'a>, EspError> {
        self.0.esp_value_mut().disconnect().await?;
        Ok(self.0)
    }

    pub fn netif(&self) -> &EspNetif {
        self.0.esp_value().wifi().sta_netif()
    }
}
