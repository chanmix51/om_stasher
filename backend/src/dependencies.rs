//! Dependencies resolution
use std::sync::Arc;

use anyhow::anyhow;
use flat_config::ConfigError;
use thiserror::Error;

use crate::{configuration::ConfigurationBuilder, ServicesContainer, StdError};

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
    db_client: Option<Arc<tokio_postgres::Client>>,
    services_container: Option<Arc<ServicesContainer>>,
    thought_store: Option<Arc<dyn crate::thoughts::model::ThoughtStore>>,
    thought_service: Option<Arc<dyn crate::thoughts::ThoughtService>>,
}

impl DependenciesBuilder {
    pub fn new(config_builder: ConfigurationBuilder) -> Self {
        Self {
            config_builder,
            db_client: None,
            services_container: None,
            thought_store: None,
            thought_service: None,
        }
    }

    async fn build_db_client(&mut self) -> Result<Arc<tokio_postgres::Client>, DependenciesError> {
        let config = self
            .config_builder
            .get_thought_config()?
            .get_database_connection_string()
            .map_err(DependenciesError::ConfigError)?;

        let (client, connection) = tokio_postgres::connect(&config, tokio_postgres::NoTls)
            .await
            .map_err(|e| DependenciesError::SetupError(e.into()))?;

        tokio::spawn(async move {
            if let Err(e) = connection.await {
                eprintln!("connection error: {}", e);
            }
        });

        Ok(Arc::new(client))
    }

    async fn get_db_client(&mut self) -> Result<Arc<tokio_postgres::Client>, DependenciesError> {
        if self.db_client.is_none() {
            self.db_client = Some(self.build_db_client().await?);
        }

        Ok(self.db_client.as_ref().cloned().unwrap())
    }

    pub async fn build_thought_store(
        &mut self,
    ) -> Result<Arc<dyn crate::thoughts::model::ThoughtStore>, DependenciesError> {
        let client = self.get_db_client().await?;
        let thought_store = crate::thoughts::model::AgrumThoughtStore::new(client);

        Ok(Arc::new(thought_store))
    }

    pub async fn get_thought_store(
        &mut self,
    ) -> Result<Arc<dyn crate::thoughts::model::ThoughtStore>, DependenciesError> {
        if self.thought_store.is_none() {
            self.thought_store = Some(self.build_thought_store().await?);
        }

        Ok(self.thought_store.as_ref().cloned().unwrap())
    }

    pub async fn build_http_runtime(
        &mut self,
    ) -> Result<Arc<crate::http::BackendHttpRuntime>, DependenciesError> {
        //let services_container = self.get_services_container().await?;
        let runtime = crate::http::BackendHttpRuntime::new(
            self.config_builder.get_http_config()?,
            self.get_services_container().await?,
        );

        Ok(Arc::new(runtime))
    }

    async fn build_thought_service(
        &mut self,
    ) -> Result<Arc<dyn crate::thoughts::ThoughtService>, DependenciesError> {
        let service = crate::thoughts::BackendThoughtService::new(
            self.config_builder.get_thought_config()?,
            self.get_thought_store().await?,
        );

        Ok(Arc::new(service))
    }

    pub async fn get_thought_service(
        &mut self,
    ) -> Result<Arc<dyn crate::thoughts::ThoughtService>, DependenciesError> {
        if self.thought_service.is_none() {
            self.thought_service = Some(self.build_thought_service().await?);
        }

        Ok(self.thought_service.as_ref().cloned().unwrap())
    }

    async fn build_services_container(
        &mut self,
    ) -> Result<Arc<ServicesContainer>, DependenciesError> {
        let thoughts_service = self.get_thought_service().await?;

        Ok(Arc::new(ServicesContainer::new(thoughts_service)))
    }

    pub async fn get_services_container(
        &mut self,
    ) -> Result<Arc<ServicesContainer>, DependenciesError> {
        if self.services_container.is_none() {
            self.services_container = Some(self.build_services_container().await?);
        }

        Ok(self.services_container.as_ref().cloned().unwrap())
    }
}
