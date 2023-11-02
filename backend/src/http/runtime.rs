//! HTTP runtime module
use std::sync::Arc;

use anyhow::anyhow;
use anyhow::Context;
use log::{debug, info};
use salvo::affix;
use salvo::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::Runtime;
use crate::{ServicesContainer, StdResult};

use super::BackendHttpConfig;

pub struct BackendHttpRuntime {
    config: Arc<BackendHttpConfig>,
    services_container: Arc<ServicesContainer>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiVersion {
    version: String,
}

impl Default for ApiVersion {
    fn default() -> Self {
        Self {
            version: "0.1.0".to_string(),
        }
    }
}

#[handler]
async fn index(
    _request: &mut Request,
    depot: &mut Depot,
    response: &mut Response,
) -> StdResult<()> {
    info!("ROUTE: index ('/').");
    let services = depot
        .obtain::<Arc<ServicesContainer>>()
        .map_err(|_| anyhow!("Could not obtain services container.".to_string()))?
        .clone();
    let thought_id = Uuid::parse_str("40b5b09f-04d3-4340-b794-c4afe9b4f6d1")?;
    let thought = services.thought_service.get_thought(&thought_id).await?;

    match thought {
        Some(t) => {
            debug!("Found thought ID='{}'.", t.thought_id);
            response.render(format!("There is a thought: {t:?}"));
        }
        None => {
            debug!("No thought found for ID='{thought_id}'.");
            response.status_code(StatusCode::NOT_FOUND);
        }
    }
    // response.render(serde_json::to_string(&ApiVersion::default()).unwrap());

    Ok(())
}

impl BackendHttpRuntime {
    pub fn new(config: Arc<BackendHttpConfig>, services_container: Arc<ServicesContainer>) -> Self {
        Self {
            config,
            services_container,
        }
    }
}

#[async_trait]
impl Runtime for BackendHttpRuntime {
    async fn run(&self) -> StdResult<()> {
        //tracing_subscriber::fmt().init();
        let router = Router::new()
            .hoop(affix::inject(self.services_container.clone()))
            .get(index);
        let acceptor = TcpListener::new(&self.config.get_listen_address())
            .try_bind()
            .await
            .with_context(|| {
                format!(
                    "Could not launch HTTP server at address '{}'.",
                    &self.config.get_listen_address(),
                )
            })?;
        Server::new(acceptor).serve(router).await;

        Ok(())
    }
}
