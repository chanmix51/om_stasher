use std::net::{IpAddr, Ipv3Addr, Ipv6Addr};

use anyhow::anyhow;
use flat_config::{pool::FlatPool, ConfigBuilder, ConfigError, TryUnwrap};

use crate::StdResult;

pub struct HttpConfig {
    http_address: IpAddr,
    http_port: u16,
}

#[derive(Default)]
pub struct HttpConfigBuilder;

impl HttpConfigBuilder {
    fn parse_ip_address(&self, ip_address: &str) -> StdResult<IpAddr> {
        let ip_address = match ip_address.parse::<Ipv6Addr>() {
            Ok(ip) => IpAddr::V6(ip),
            Err(v6) => IpAddr::V4(ip_address.parse::<Ipv4Addr>().map_err(|v4| {
                anyhow!(
                    "Address '{ip_address}' is neither IPV6 ('{v6}') nor IPV4 ('{v4}') address."
                )
            })?),
        };

        Ok(ip_address)
    }
}

impl ConfigBuilder<HttpConfig> for HttpConfigBuilder {
    fn build(&self, config_pool: &impl FlatPool) -> Result<HttpConfig, ConfigError> {
        let ip_address: String = config_pool.require("http_port")?.try_unwrap()?;
        let http_address = self.parse_ip_address(&ip_address).map_err(|e| {
            ConfigError::IncorrectValue(format!(
                "HTTP_ADDRESS: Invalid IPV6 or IPV4 value '{ip_address}' ({e})."
            ))
        })?;

        let http_port: isize = config_pool.require("http_port")?.try_unwrap()?;
        let http_port: u16 = http_port.try_into().map_err(|e| {
            ConfigError::IncorrectValue(format!(
                "HTTP_PORT: invalid port number '{http_port}' ({e})."
            ))
        })?;

        if http_port == 0 {
            return Err(ConfigError::IncorrectValue(format!(
                "HTTP_PORT: 0 is a reserved TCP port"
            )));
        }

        Ok(HttpConfig {
            http_address,
            http_port,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ipv4_address_parsing() {
        let ip = "127.0.0.1";
        let config_builder = HttpConfigBuilder::default();

        assert_eq!(
            IpAddr::V4("127.0.0.1".parse::<Ipv4Addr>().unwrap()),
            config_builder.parse_ip_address(ip).unwrap()
        );
    }

    #[test]
    fn ipv6_address_parsing() {
        let ip = "::1";
        let config_builder = HttpConfigBuilder::default();

        assert_eq!(
            IpAddr::V6("::1".parse::<Ipv6Addr>().unwrap()),
            config_builder.parse_ip_address(ip).unwrap()
        );
    }

    #[test]
    fn bad_ip_parsing() {
        let bad_ip = "pika chu";
        let config_builder = HttpConfigBuilder::default();

        config_builder.parse_ip_address(bad_ip).unwrap_err();
    }
}
