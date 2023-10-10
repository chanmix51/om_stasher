use std::sync::Arc;

use async_trait::async_trait;
use thiserror::Error;

use crate::StdResult;

use super::{
    model::{ThoughtEnvelope, ThoughtSource, ThoughtStore},
    ThoughtServiceConfig,
};

#[derive(Debug, Error)]
pub enum ThoughtServiceError {
    #[error("Parent node '{0}' does not exist")]
    ParentNodeDoesNotExist(String),
}

/// Description of the API for BackendHttpService`
#[async_trait]
pub trait ThoughtService {
    /// Retrieve a thought from the referential, if no thought is found, None is returned.
    async fn get_thought(&self, thought_id: &str) -> StdResult<Option<ThoughtEnvelope>>;

    /// Create or update a Thought. It raises an `ThoughtServiceError::ParentNodeDoesNotExist` if
    /// the given `parent_thought_id` does not exist.  If no `parent_thought_id` is given, a new
    /// `Thread` is created.
    async fn post_thought(
        &self,
        thought: String,
        parent_thought_id: Option<String>,
        keywords: Vec<String>,
        categories: Vec<String>,
        sources: Vec<ThoughtSource>,
    ) -> StdResult<ThoughtEnvelope>;

    /// Retrieve a Thread from the referential, the given thought_id is one the the Thread's
    /// thought. The Thread is returned from the first thought to the thought pointed by the given
    /// thought_id, if it does not exist, None is returned.
    async fn get_thread(&self, thought_id: &str) -> StdResult<Option<Vec<ThoughtEnvelope>>>;
}

pub struct BackendThoughtService {
    config: Arc<ThoughtServiceConfig>,
    thought_store: Arc<dyn ThoughtStore>,
}

impl BackendThoughtService {
    pub fn new(config: Arc<ThoughtServiceConfig>, thought_store: Arc<dyn ThoughtStore>) -> Self {
        Self {
            config,
            thought_store,
        }
    }
}

#[async_trait]
impl ThoughtService for BackendThoughtService {
    async fn get_thread(&self, thought_id: &str) -> StdResult<Option<Vec<ThoughtEnvelope>>> {
        todo!()
    }

    async fn post_thought(
        &self,
        thought: String,
        parent_thought_id: Option<String>,
        keywords: Vec<String>,
        categories: Vec<String>,
        sources: Vec<ThoughtSource>,
    ) -> StdResult<ThoughtEnvelope> {
        todo!()
    }

    async fn get_thought(&self, thought_id: &str) -> StdResult<Option<ThoughtEnvelope>> {
        todo!()
    }
}
