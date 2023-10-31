use std::sync::Arc;

use flat_config::{pool::SimpleFlatPool, ConfigBuilder, ConfigError};
use tokio::sync::OnceCell;

type DynFlatPool = SimpleFlatPool;

/// Configuration builder
pub struct ConfigurationBuilder {
    flat_pool: DynFlatPool,
    http_config: OnceCell<Arc<crate::http::BackendHttpConfig>>,
    thought_config: OnceCell<Arc<crate::thoughts::ThoughtServiceConfig>>,
}

impl ConfigurationBuilder {
    pub fn new(flat_pool: DynFlatPool) -> Self {
        Self {
            flat_pool,
            http_config: OnceCell::new(),
            thought_config: OnceCell::new(),
        }
    }

    async fn build_http_config(&self) -> Result<Arc<crate::http::BackendHttpConfig>, ConfigError> {
        let config = crate::http::BackendHttpConfigBuilder {}.build(&self.flat_pool)?;

        Ok(Arc::new(config))
    }

    pub async fn get_http_config(
        &self,
    ) -> Result<Arc<crate::http::BackendHttpConfig>, ConfigError> {
        let init = self.build_http_config();

        self.http_config
            .get_or_try_init(|| init)
            .await
            .map(|x| x.clone())
    }

    async fn build_thought_config(
        &self,
    ) -> Result<Arc<crate::thoughts::ThoughtServiceConfig>, ConfigError> {
        let config = crate::thoughts::ThoughtServiceConfigBuilder {}.build(&self.flat_pool)?;

        Ok(Arc::new(config))
    }

    pub async fn get_thought_config(
        &self,
    ) -> Result<Arc<crate::thoughts::ThoughtServiceConfig>, ConfigError> {
        let init = self.build_thought_config();

        self.thought_config
            .get_or_try_init(|| init)
            .await
            .map(|x| x.clone())
    }
}
