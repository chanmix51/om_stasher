use std::{borrow::Borrow, sync::Arc};

use agrum::core::Provider;
use async_trait::async_trait;
use tokio_postgres::Client;
use uuid::Uuid;

use crate::StdResult;

use super::{
    agrum::{ThoughtEntityRepository, ThoughtEntitySqlDefinition},
    ThoughtEnvelope as Thought,
};

/// The ThoughtStore is responsible of offering a generic API to persist and retreive thought
/// entities. It also configures the way the thoughts are being fetch and the kind of thought
/// entities returned by the different queries. The `SqlEntity` instances shall not being exposed
/// outside the store.
#[async_trait]
pub trait ThoughtStore: Sync + Send {
    async fn get_thought(&self, thought_id: &Uuid) -> StdResult<Option<Thought>>;
}

pub struct AgrumThoughtStore {
    client: Arc<Client>,
}

impl AgrumThoughtStore {
    /// Constructor
    pub fn new(client: Arc<Client>) -> Self {
        Self { client }
    }
}

#[async_trait]
impl ThoughtStore for AgrumThoughtStore {
    async fn get_thought(&self, thought_id: &Uuid) -> StdResult<Option<Thought>> {
        let thought_repository = ThoughtEntityRepository::new(Provider::new(
            self.client.borrow(),
            Box::new(ThoughtEntitySqlDefinition::default()),
        ));

        thought_repository
            .get_thought(thought_id)
            .await
            .map(|o| o.map(|t| t.into()))
    }
}
