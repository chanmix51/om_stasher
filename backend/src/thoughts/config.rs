use dsn::DSN;
use flat_config::{pool::FlatPool, ConfigBuilder, ConfigError, TryUnwrap};

use crate::StdResult;

pub struct ThoughtServiceConfig {
    database_dsn: DSN,
}

impl ThoughtServiceConfig {
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
pub struct ThoughtServiceConfigBuilder;

impl ConfigBuilder<ThoughtServiceConfig> for ThoughtServiceConfigBuilder {
    fn build(&self, config_pool: &impl FlatPool) -> Result<ThoughtServiceConfig, ConfigError> {
        let dsn_string: String = config_pool.require("database_dsn")?.try_unwrap()?;
        let database_dsn = dsn::parse(&dsn_string).map_err(|e| {
            ConfigError::IncorrectValue(format!(
                "Invalid database DSN: '{dsn_string}', parser said: {e}."
            ))
        })?;

        Ok(ThoughtServiceConfig { database_dsn })
    }
}

#[cfg(test)]
mod tests {
    use flat_config::pool::SimpleFlatPool;

    use super::*;

    #[test]
    fn test_connection_dsn_without_password() -> StdResult<()> {
        let mut flat_pool = SimpleFlatPool::default();
        flat_pool.add("database_dsn", "pgsql://user@host".into());
        let config = ThoughtServiceConfigBuilder::default()
            .build(&flat_pool)
            .unwrap();

        assert_eq!(
            "host=host user=user",
            &config.get_database_connection_string()?
        );

        Ok(())
    }

    #[test]
    fn test_connection_dsn_with_password() -> StdResult<()> {
        let mut flat_pool = SimpleFlatPool::default();
        flat_pool.add("database_dsn", "pgsql://user:passw@host".into());
        let config = ThoughtServiceConfigBuilder::default()
            .build(&flat_pool)
            .unwrap();

        assert_eq!(
            "host=host user=user password=passw",
            &config.get_database_connection_string()?
        );

        Ok(())
    }
}
