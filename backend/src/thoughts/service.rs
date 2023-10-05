use std::sync::Arc;

use thiserror::Error;

use crate::StdResult;

use super::{
    model::{ThoughtEnvelope, ThoughtSource},
    ThoughtsServiceConfig,
};

#[derive(Debug, Error)]
pub enum ThoughtsServiceError {
    #[error("Parent node '{0}' does not exist")]
    ParentNodeDoesNotExist(String),
}

/// Description of the API for BackendHttpService`
pub trait ThoughtsService {
    /// Retrieve a thought from the referential, if no thought is found, None is returned.
    fn get_thought(&self, thought_id: &str) -> StdResult<Option<ThoughtEnvelope>>;

    /// Create or update a Thought. It raises an `ThoughtsServiceError::ParentNodeDoesNotExist` if
    /// the given `parent_thought_id` does not exist.  If no `parent_thought_id` is given, a new
    /// `Thread` is created.
    fn post_thought(
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
    fn get_thread(&self, thought_id: &str) -> StdResult<Option<Vec<ThoughtEnvelope>>>;
}

pub struct BackendThoughtsService {
    config: Arc<ThoughtsServiceConfig>,
}

impl BackendThoughtsService {
    pub fn new(config: Arc<ThoughtsServiceConfig>) -> Self {
        Self { config }
    }
}
