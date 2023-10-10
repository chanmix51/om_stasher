use std::sync::Arc;

use async_trait::async_trait;
use uuid::Uuid;

use crate::StdResult;

use super::{agrum::ThoughtEntityRepository, ThoughtEnvelope as Thought};

#[async_trait]
pub trait ThoughtStore: Sync + Send {
    async fn get_thought(&self, thought_id: &Uuid) -> StdResult<Option<Thought>>;
}

pub struct AgrumThoughtStore<'client> {
    thought_provider: Arc<ThoughtEntityRepository<'client>>,
}

impl<'client> AgrumThoughtStore<'client> {
    pub fn new(thought_provider: Arc<ThoughtEntityRepository<'client>>) -> Self {
        Self { thought_provider }
    }
}
