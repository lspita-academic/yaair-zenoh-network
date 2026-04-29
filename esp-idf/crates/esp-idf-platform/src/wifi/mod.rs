use std::ops::{Deref, DerefMut};

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

impl<'a> Deref for Wifi<'a> {
    type Target = WifiImpl<'a>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Wifi<'_>  {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
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
        &mut self,
        config: &WifiConfig,
    ) -> Result<WifiConnection<'a, '_>, EspError> {
        self.set_configuration(config)?;
        self.start().await?;
        log::info!("Wifi started");

        self.connect().await?;
        log::info!("Wifi connected");

        self.wait_netif_up().await?;
        log::info!("Wifi netif up");
        Ok(WifiConnection(self))
    }
}

pub struct WifiConnection<'a, 'b>(&'b mut Wifi<'a>);

impl<'a> WifiConnection<'a, '_> {
    pub async fn disconnect(self) -> Result<(), EspError> {
        self.0.disconnect().await
    }

    pub fn netif(&self) -> &EspNetif {
        self.0.wifi().sta_netif()
    }
}
