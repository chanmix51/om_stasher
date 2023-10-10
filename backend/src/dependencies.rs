//! Dependencies resolution

use anyhow::anyhow;
use flat_config::ConfigError;
use std::sync::Arc;
use thiserror::Error;

use crate::{configuration::ConfigurationBuilder, StdError};

#[derive(Error, Debug)]
pub enum DependenciesError {
    #[error("Dependency configuration error: {0}")]
    ConfigError(StdError),

    #[error("Dependency setup error: {0}")]
    SetupError(StdError),
}

impl From<ConfigError> for DependenciesError {
    fn from(value: ConfigError) -> Self {
        Self::ConfigError(anyhow!(value))
    }
}

pub struct DependenciesBuilder {
    config_builder: ConfigurationBuilder,
}

impl DependenciesBuilder {
    pub fn new(config_builder: ConfigurationBuilder) -> Self {
        Self { config_builder }
    }

    pub async fn build_sql_client(&mut self) -> Result<tokio_postgres::Client, DependenciesError> {
        /*
        let config = self
            .configuration
            .get_postgres_connection_string()
            .map_err(DependenciesError::ConfigError)?;

        let (client, connection) = tokio_postgres::connect(&config, tokio_postgres::NoTls)
            .await
            .map_err(|e| DependenciesError::SetupError(e.into()))?;

        tokio::spawn(async move {
            if let Err(e) = connection.await {
                eprintln!("connection error: {}", e);
            }
        });

        Ok(client)
        */
        todo!()
    }

    pub async fn build_http_service(
        &mut self,
    ) -> Result<Arc<crate::http::BackendHttpService>, DependenciesError> {
        let service = crate::http::BackendHttpService::new(self.config_builder.get_http_config()?);

        Ok(Arc::new(service))
    }

    pub async fn build_thought_service(
        &mut self,
    ) -> Result<Arc<crate::thoughts::BackendThoughtsService>, DependenciesError> {
        let service =
            crate::thoughts::BackendThoughtsService::new(self.config_builder.get_thought_config()?);

        Ok(Arc::new(service))
    }
}
