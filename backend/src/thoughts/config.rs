use dsn::DSN;
use flat_config::{pool::FlatPool, ConfigBuilder, ConfigError, TryUnwrap};

use crate::StdResult;

pub struct ThoughtsServiceConfig {
    database_dsn: DSN,
}

impl ThoughtsServiceConfig {
    pub fn get_database_connection_string(&self) -> StdResult<String> {
        let connstring = format!(
            "host={} user={}",
            self.database_dsn.host.as_ref().unwrap(),
            self.database_dsn.username.as_ref().unwrap()
        );

        let connstring = if let Some(password) = &self.database_dsn.password {
            format!("{connstring} password={password}")
        } else {
            connstring
        };

        Ok(connstring)
    }
}

#[derive(Debug, Default)]
pub struct ThoughtsServiceConfigBuilder;

impl ConfigBuilder<ThoughtsServiceConfig> for ThoughtsServiceConfigBuilder {
    fn build(&self, config_pool: &impl FlatPool) -> Result<ThoughtsServiceConfig, ConfigError> {
        let dsn_string: String = config_pool.require("database_dsn")?.try_unwrap()?;
        let database_dsn = dsn::parse(&dsn_string).map_err(|e| {
            ConfigError::IncorrectValue(format!(
                "Invalid database DSN: '{dsn_string}', parser said: {e}."
            ))
        })?;

        Ok(ThoughtsServiceConfig { database_dsn })
    }
}

#[cfg(test)]
mod tests {
    use flat_config::pool::SimpleFlatPool;

    use super::*;

    #[test]
    fn test_connection_dsn_without_password() {
        let mut flat_pool = SimpleFlatPool::default();
        flat_pool.add("database_dsn", "pgsql://user@host".into());
        let config = ThoughtsServiceConfigBuilder::default()
            .build(flat_pool)
            .unwrap();

        assert_eq!("host=host user=user",, &config.get_database_connection_string());
    }

    #[test]
    fn test_connection_dsn_with_password() {
        let mut flat_pool = SimpleFlatPool::default();
        flat_pool.add("database_dsn", "pgsql://user:passw@host".into());
        let config = ThoughtsServiceConfigBuilder::default()
            .build(flat_pool)
            .unwrap();

        assert_eq!("host=host user=user password=passw",, &config.get_database_connection_string());
    }
}
