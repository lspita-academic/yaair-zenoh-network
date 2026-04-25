use std::{fmt::Display, net::SocketAddrV4};

use strum::{Display, EnumString};

#[derive(Debug, Display, EnumString)]
#[strum(serialize_all = "snake_case")]
pub enum LocatorProtocol {
    UDP,
    TCP,
}

pub struct Locator {
    pub protocol: LocatorProtocol,
    pub endpoint: SocketAddrV4,
    pub iface: Option<String>,
}

impl Display for Locator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}/{}:{}",
            self.protocol,
            self.endpoint.ip(),
            self.endpoint.port()
        )?;
        if let Some(iface) = &self.iface {
            write!(f, "#iface={iface}")
        } else {
            Ok(())
        }
    }
}
