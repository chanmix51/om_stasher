use chrono::{DateTime, Utc};
use either::Either;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::ThoughtEntity;

#[derive(Debug, Serialize, Deserialize)]
pub struct ThoughtSource {
    pub name: String,
    pub authors: Vec<String>,
    pub description: String,
}

#[derive(Debug)]
pub struct ThoughtNode {
    pub parent_thought_id: Uuid,
    pub thought: String,
}

#[derive(Debug)]
pub struct ThoughtThread {
    pub title: String,
}

#[derive(Debug)]
pub struct ThoughtEnvelope {
    pub thought_id: Uuid,
    pub keywords: Vec<String>,
    pub categories: Vec<String>,
    pub sources: Vec<ThoughtSource>,
    pub created_at: DateTime<Utc>,
    pub content: Either<ThoughtThread, ThoughtNode>,
}

impl From<ThoughtEntity> for ThoughtEnvelope {
    fn from(value: ThoughtEntity) -> Self {
        let content = if let Some(parent_thought_id) = value.parent_thought_id {
            let thought_node = ThoughtNode {
                parent_thought_id,
                thought: value.content,
            };

            Either::Right(thought_node)
        } else {
            Either::Left(ThoughtThread {
                title: value.content,
            })
        };

        let sources: Vec<ThoughtSource> = value
            .sources
            .into_iter()
            .map(|c| -> ThoughtSource { serde_json::from_str(&c).unwrap() })
            .collect();

        Self {
            thought_id: value.thought_id,
            keywords: value.keywords,
            categories: value.categories,
            sources,
            created_at: value.created_at,
            content,
        }
    }
}
