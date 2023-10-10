use std::sync::Arc;

use flat_config::{pool::SimpleFlatPool, ConfigBuilder, ConfigError};

type DynFlatPool = SimpleFlatPool;

/// Configuration builder
pub struct ConfigurationBuilder {
    flat_pool: DynFlatPool,
    http_config: Option<Arc<crate::http::BackendHttpConfig>>,
    thought_config: Option<Arc<crate::thoughts::ThoughtsServiceConfig>>,
}

impl ConfigurationBuilder {
    pub fn new(flat_pool: DynFlatPool) -> Self {
        Self {
            flat_pool,
            http_config: None,
            thought_config: None,
        }
    }

    fn build_http_config(&self) -> Result<Arc<crate::http::BackendHttpConfig>, ConfigError> {
        let config = crate::http::BackendHttpConfigBuilder {}.build(&self.flat_pool)?;

        Ok(Arc::new(config))
    }

    pub fn get_http_config(&mut self) -> Result<Arc<crate::http::BackendHttpConfig>, ConfigError> {
        if self.http_config.is_none() {
            self.http_config = Some(self.build_http_config()?);
        }

        Ok(self.http_config.as_ref().cloned().unwrap())
    }

    fn build_thought_config(
        &mut self,
    ) -> Result<Arc<crate::thoughts::ThoughtsServiceConfig>, ConfigError> {
        let config = crate::thoughts::ThoughtsServiceConfigBuilder {}.build(&self.flat_pool)?;

        Ok(Arc::new(config))
    }

    pub fn get_thought_config(
        &mut self,
    ) -> Result<Arc<crate::thoughts::ThoughtsServiceConfig>, ConfigError> {
        if self.thought_config.is_none() {
            self.thought_config = Some(self.build_thought_config()?);
        }

        Ok(self.thought_config.as_ref().cloned().unwrap())
    }
}
