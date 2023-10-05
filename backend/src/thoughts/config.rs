use flat_config::{pool::FlatPool, ConfigBuilder, ConfigError, TryUnwrap};

pub struct ThoughtsServiceConfig {
    database_dsn: String,
}

pub struct ThoughtsServiceConfigBuilder;

impl ConfigBuilder<ThoughtsServiceConfig> for ThoughtsServiceConfigBuilder {
    fn build(&self, config_pool: &impl FlatPool) -> Result<ThoughtsServiceConfig, ConfigError> {
        let database_dsn = config_pool.require("database_dsn")?.try_unwrap()?;

        Ok(ThoughtsServiceConfig { database_dsn })
    }
}
