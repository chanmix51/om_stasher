use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::agrum::ThoughtEntity;

#[derive(Debug, Serialize, Deserialize)]
pub struct ThoughtSource {
    pub name: String,
    pub authors: Vec<String>,
    pub description: String,
}

#[derive(Debug)]
pub enum ThoughtContent {
    Node {
        parent_thought_id: Uuid,
        thought: String,
    },
    Thread {
        title: String,
    },
}

#[derive(Debug)]
pub struct ThoughtEnvelope {
    pub thought_id: Uuid,
    pub keywords: Vec<String>,
    pub categories: Vec<String>,
    pub sources: Vec<ThoughtSource>,
    pub created_at: DateTime<Utc>,
    pub content: ThoughtContent,
}

impl From<ThoughtEntity> for ThoughtEnvelope {
    fn from(value: ThoughtEntity) -> Self {
        let content = if let Some(parent_thought_id) = value.parent_thought_id {
            ThoughtContent::Node {
                parent_thought_id,
                thought: value.content,
            }
        } else {
            ThoughtContent::Thread {
                title: value.content,
            }
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
