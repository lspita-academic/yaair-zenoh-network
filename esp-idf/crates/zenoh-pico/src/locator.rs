use std::{fmt::Display, net::SocketAddrV4};

#[derive(Debug, Default)]
pub enum LocatorProtocol {
    #[default]
    UDP,
    TCP,
}

impl Display for LocatorProtocol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::UDP => "udp",
            Self::TCP => "tcp",
        })
    }
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
