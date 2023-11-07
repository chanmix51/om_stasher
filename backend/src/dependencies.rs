//! Dependencies resolution
use std::sync::Arc;

use anyhow::anyhow;
use flat_config::ConfigError;
use log::{debug, error, trace};
use thiserror::Error;
use tokio::sync::{Mutex, OnceCell};

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
    db_client: OnceCell<Arc<tokio_postgres::Client>>,
    services_container: OnceCell<Arc<ServicesContainer>>,
    thought_store: OnceCell<Arc<dyn crate::thoughts::model::ThoughtStore>>,
    thought_service: OnceCell<Arc<dyn crate::thoughts::ThoughtService>>,
    event_dispatcher: OnceCell<Arc<crate::EventDispatcher>>,
}

impl DependenciesBuilder {
    pub fn new(config_builder: ConfigurationBuilder) -> Self {
        Self {
            config_builder,
            db_client: OnceCell::new(),
            services_container: OnceCell::new(),
            thought_store: OnceCell::new(),
            thought_service: OnceCell::new(),
            event_dispatcher: OnceCell::new(),
        }
    }

    async fn build_db_client(&self) -> Result<Arc<tokio_postgres::Client>, DependenciesError> {
        trace!("DEP BUILDER: build database connection…");
        let connection_string = self
            .config_builder
            .get_thought_config()
            .await?
            .get_database_connection_string()
            .map_err(DependenciesError::ConfigError)?;
        debug!("Database connection string: '{connection_string}'");

        let (client, connection) =
            tokio_postgres::connect(&connection_string, tokio_postgres::NoTls)
                .await
                .map_err(|e| {
                    DependenciesError::SetupError(anyhow!(e).context(format!(
                        "Error opening database connection using string '{connection_string}'."
                    )))
                })?;

        tokio::spawn(async move {
            if let Err(e) = connection.await {
                error!("Database connection error: {}", e);
            } else {
                trace!("Database connection closed.");
            }
        });

        Ok(Arc::new(client))
    }

    async fn get_db_client(&self) -> Result<Arc<tokio_postgres::Client>, DependenciesError> {
        trace!("DEP BUILDER: get database connection…");
        let init = self.build_db_client();

        self.db_client
            .get_or_try_init(|| init)
            .await
            .map(|x| x.clone())
    }

    async fn build_thought_store(
        &self,
    ) -> Result<Arc<dyn crate::thoughts::model::ThoughtStore>, DependenciesError> {
        trace!("DEP BUILDER: build thought store…");
        let client = self.get_db_client().await?;
        let thought_store = crate::thoughts::model::AgrumThoughtStore::new(client);

        Ok(Arc::new(thought_store))
    }

    pub async fn get_thought_store(
        &self,
    ) -> Result<Arc<dyn crate::thoughts::model::ThoughtStore>, DependenciesError> {
        trace!("DEP BUILDER: get thought store…");
        let init = self.build_thought_store();

        self.thought_store
            .get_or_try_init(|| init)
            .await
            .map(|x| x.clone())
    }

    pub async fn build_http_runtime(
        &self,
    ) -> Result<Arc<crate::http::BackendHttpRuntime>, DependenciesError> {
        trace!("DEP BUILDER: build HTTP server runtime…");
        let runtime = crate::http::BackendHttpRuntime::new(
            self.config_builder.get_http_config().await?,
            self.get_services_container().await?,
        );

        Ok(Arc::new(runtime))
    }

    pub async fn build_thought_runtime(
        &self,
    ) -> Result<Arc<crate::Runtime<crate::thoughts::ThoughtServiceRuntime>>, DependenciesError>
    {
        trace!("DEP BUILDER: build Thought runtime…");
        let service_runtime =
            crate::thoughts::ThoughtServiceRuntime::new(self.get_services_container().await?);
        let (_, broadcast_receiver) = self.get_event_dispatcher().await?.subscribe();

        Ok(Arc::new(crate::Runtime::new(
            Arc::new(service_runtime),
            Arc::new(Mutex::new(broadcast_receiver)),
        )))
    }

    pub async fn build_logger_runtime(
        &self,
    ) -> Result<Arc<crate::Runtime<crate::LoggerServiceRuntime>>, DependenciesError> {
        trace!("DEP BUILDER: build event logger runtime…");
        let service_runtime = crate::LoggerServiceRuntime;
        let (_, broadcast_receiver) = self.get_event_dispatcher().await?.subscribe();

        Ok(Arc::new(crate::Runtime::new(
            Arc::new(service_runtime),
            Arc::new(Mutex::new(broadcast_receiver)),
        )))
    }

    async fn build_event_dispatcher(
        &self,
    ) -> Result<Arc<crate::EventDispatcher>, DependenciesError> {
        trace!("DEP BUILDER: build event dispatcher…");
        let dispatcher = crate::EventDispatcher::default();

        Ok(Arc::new(dispatcher))
    }

    pub async fn get_event_dispatcher(
        &self,
    ) -> Result<Arc<crate::EventDispatcher>, DependenciesError> {
        trace!("DEP BUILDER: get event dispatcher…");
        let init = self.build_event_dispatcher();

        self.event_dispatcher
            .get_or_try_init(|| init)
            .await
            .map(|x| x.clone())
    }

    async fn build_thought_service(
        &self,
    ) -> Result<Arc<dyn crate::thoughts::ThoughtService>, DependenciesError> {
        trace!("DEP BUILDER: build Thought service…");
        let (sender, _) = self.get_event_dispatcher().await?.subscribe();
        let sender = Arc::new(Mutex::new(sender));

        let service = crate::thoughts::BackendThoughtService::new(
            self.config_builder.get_thought_config().await?,
            self.get_thought_store().await?,
            sender,
        );

        Ok(Arc::new(service))
    }

    pub async fn get_thought_service(
        &self,
    ) -> Result<Arc<dyn crate::thoughts::ThoughtService>, DependenciesError> {
        trace!("DEP BUILDER: get Thought service…");
        let init = self.build_thought_service();

        self.thought_service
            .get_or_try_init(|| init)
            .await
            .map(|x| x.clone())
    }

    async fn build_services_container(&self) -> Result<Arc<ServicesContainer>, DependenciesError> {
        trace!("DEP BUILDER: build services container…");
        let thoughts_service = self.get_thought_service().await?;

        Ok(Arc::new(ServicesContainer::new(thoughts_service)))
    }

    pub async fn get_services_container(
        &self,
    ) -> Result<Arc<ServicesContainer>, DependenciesError> {
        trace!("DEP BUILDER: get services container…");
        let init = self.build_services_container();

        self.services_container
            .get_or_try_init(|| init)
            .await
            .map(|x| x.clone())
    }
}
