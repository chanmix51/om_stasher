//! HTTP service module
use std::sync::Arc;

use salvo::prelude::*;
use serde::{Deserialize, Serialize};

use crate::StdResult;

use super::BackendHttpConfig;

pub struct BackendHttpService {
    config: Arc<BackendHttpConfig>,
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
async fn index(res: &mut Response) {
    res.render(serde_json::to_string(&ApiVersion::default()).unwrap());
}

impl BackendHttpService {
    pub fn new(config: Arc<BackendHttpConfig>) -> Self {
        Self { config }
    }

    pub async fn run(&self) -> StdResult<()> {
        //tracing_subscriber::fmt().init();
        let router = Router::new().get(index);
        let acceptor = TcpListener::new(&self.config.get_listen_address())
            .bind()
            .await;
        Server::new(acceptor).serve(router).await;

        Ok(())
    }
}
