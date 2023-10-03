//! Dependencies resolution

use anyhow::Error;
use std::{fmt::Display, sync::Arc};
use thiserror::Error;

use flat_config::pool::FlatPool;

use crate::StdResult;

#[derive(Error, Debug)]
pub enum DependenciesError {
    #[error("Dependency configuration error: {0}")]
    ConfigError(Error),
}

pub struct Dependencies {
    configuration: Arc<dyn FlatPool>,
}

impl Dependencies {
    pub fn new(configuration: Arc<dyn FlatPool>) -> Self {
        Self { configuration }
    }
}
