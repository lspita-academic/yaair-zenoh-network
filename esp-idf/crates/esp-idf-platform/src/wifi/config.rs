use esp_idf_svc::wifi::{ClientConfiguration, Configuration};
use heapless::CapacityError;
use thiserror::Error;

pub struct WifiConfig(Configuration);

impl From<Configuration> for WifiConfig {
    fn from(value: Configuration) -> Self {
        Self(value)
    }
}

#[derive(Debug, Error)]
pub enum WifiConfigError {
    #[error("Missing required env var: {0}")]
    MissingEnvVar(String),
    #[error("SSID too long: {0}")]
    SSIDTooLong(CapacityError),
    #[error("Password too long: {0}")]
    PasswordTooLong(CapacityError),
}

macro_rules! option_env_with_err {
    ($name:literal) => {
        option_env!($name).ok_or(WifiConfigError::MissingEnvVar($name.to_owned()))
    };
}

impl WifiConfig {
    pub fn try_from_comptime_env() -> Result<Self, WifiConfigError> {
        let ssid = option_env_with_err!("ESP_WIFI_SSID")?;
        let password = option_env_with_err!("ESP_WIFI_PASSWORD")?;

        Ok(Self(
            Configuration::Client(ClientConfiguration {
                ssid: ssid.try_into().map_err(WifiConfigError::SSIDTooLong)?,
                password: password
                    .try_into()
                    .map_err(WifiConfigError::PasswordTooLong)?,
                ..Default::default()
            })
            .into(),
        ))
    }

    pub(super) fn esp_value(&self) -> &Configuration {
        &self.0
    }
}
