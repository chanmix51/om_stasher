use std::sync::Arc;

use async_trait::async_trait;
use log::trace;
use thiserror::Error;
use uuid::Uuid;

use crate::StdResult;

use super::{
    model::{agrum::ThoughtEntity, ThoughtEnvelope, ThoughtSource, ThoughtStore},
    ThoughtServiceConfig,
};

#[derive(Debug, Error)]
pub enum ThoughtServiceError {
    #[error("Parent node '{0}' does not exist")]
    ParentNodeDoesNotExist(String),
}

/// Description of the API for BackendHttpService`
#[async_trait]
pub trait ThoughtService: Sync + Send {
    /// Retrieve a thought from the referential, if no thought is found, None is returned.
    async fn get_thought(&self, thought_id: &Uuid) -> StdResult<Option<ThoughtEnvelope>>;

    /// Create or update a Thought. It raises an `ThoughtServiceError::ParentNodeDoesNotExist` if
    /// the given `parent_thought_id` does not exist.  If no `parent_thought_id` is given, a new
    /// `Thread` is created.
    async fn post_thought(
        &self,
        thought_id: String,
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
        trace!("THOUGHT SERVICE: get_thread({thought_id})");
        todo!()
    }

    async fn post_thought(
        &self,
        thought_id: String,
        parent_thought_id: Option<String>,
        keywords: Vec<String>,
        categories: Vec<String>,
        sources: Vec<ThoughtSource>,
    ) -> StdResult<ThoughtEnvelope> {
        trace!("THOUGHT SERVICE: post_thought(thought_id='{thought_id}')");
        todo!()
    }

    async fn get_thought(&self, thought_id: &Uuid) -> StdResult<Option<ThoughtEnvelope>> {
        trace!("THOUGHT SERVICE: get_thought({thought_id})");
        let thought = ThoughtEntity {
            thought_id: thought_id.to_owned(),
            parent_thought_id: None,
            keywords: vec!["keyword_a".to_string(), "keyword_b".to_string()],
            categories: vec!["cat_1".to_string(), "cat_2".to_string()],
            sources: vec![
                r#"{"name": "source_name", "authors": ["source_author"], "description": "source description"}"#.to_string(),
            ],
            created_at: chrono::DateTime::UNIX_EPOCH,
            content: r#"This is the thought content"#.to_string(),
        };

        Ok(Some(thought.into()))
    }
}
