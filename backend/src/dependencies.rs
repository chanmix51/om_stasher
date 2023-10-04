//! Dependencies resolution

use anyhow::Error;
use flat_config::{pool::SimpleFlatPool, ConfigBuilder};
use std::sync::Arc;
use thiserror::Error;

use crate::http::{BackendHttpConfigBuilder, BackendHttpService};

#[derive(Error, Debug)]
pub enum DependenciesError {
    #[error("Dependency configuration error: {0}")]
    ConfigError(Error),
}

pub struct Dependencies {
    configuration: SimpleFlatPool,
}

impl Dependencies {
    pub fn new(configuration: SimpleFlatPool) -> Self {
        Self { configuration }
    }

    pub async fn build_http_service(
        &mut self,
    ) -> Result<Arc<BackendHttpService>, DependenciesError> {
        let config = BackendHttpConfigBuilder {}
            .build(&self.configuration)
            .map_err(|e| DependenciesError::ConfigError(e.into()))?;
        let service = BackendHttpService::new(Arc::new(config));

        Ok(Arc::new(service))
    }
}
