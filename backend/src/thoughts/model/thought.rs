use chrono::{DateTime, Utc};
use either::Either;

pub struct ThoughtSource {
    pub name: String,
    pub authors: Vec<String>,
    pub description: String,
}

pub struct ThoughtNode {
    pub parent_thought_id: String,
    pub thought: String,
}

pub struct ThoughtThread {
    pub title: String,
}

pub struct ThoughtEnvelope {
    pub thought_id: String,
    pub keywords: Vec<String>,
    pub categories: Vec<String>,
    pub sources: Vec<ThoughtSource>,
    pub created_at: DateTime<Utc>,
    pub content: Either<ThoughtThread, ThoughtNode>,
}
