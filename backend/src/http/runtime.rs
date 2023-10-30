//! HTTP runtime module
use std::sync::Arc;

use anyhow::anyhow;
use salvo::affix;
use salvo::prelude::*;
use serde::{Deserialize, Serialize};

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
    let services = depot
        .obtain::<Arc<ServicesContainer>>()
        .map_err(|_| anyhow!("Could not obtain services container.".to_string()))?
        .clone();
    let thought = services.thought_service.get_thought("whatever").await?;

    match thought {
        Some(t) => {
            response.render(Text::Plain(format!("There is a thought: {t:?}")));
        }
        None => {
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

    pub async fn run(&self) -> StdResult<()> {
        //tracing_subscriber::fmt().init();
        let router = Router::new()
            .hoop(affix::inject(self.services_container.clone()))
            .get(index);
        let acceptor = TcpListener::new(&self.config.get_listen_address())
            .bind()
            .await;
        Server::new(acceptor).serve(router).await;

        Ok(())
    }
}
