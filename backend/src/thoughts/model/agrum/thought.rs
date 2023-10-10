use agrum::{
    core::{
        HydrationError, Projection, Provider, SourceAliases, SqlDefinition, SqlEntity, Structure,
        Structured, WhereCondition,
    },
    params,
};
use anyhow::anyhow;
use chrono::{DateTime, Utc};
use tokio_postgres::Row;
use uuid::Uuid;

use crate::StdResult;

/// Entity read/written from/to database.
pub struct ThoughtEntity {
    pub thought_id: Uuid,
    pub parent_thought_id: Option<Uuid>,
    pub keywords: Vec<String>,
    pub categories: Vec<String>,
    pub sources: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub content: String,
}

impl Structured for ThoughtEntity {
    fn get_structure() -> Structure {
        Structure::new(&[
            ("thought_id", "text"),
            ("parent_thought_id", "text"),
            ("keywords", "text[]"),
            ("categories", "text[]"),
            ("sources", "jsonb[]"),
            ("created_at", "timestamp"),
            ("content", "jsonb"),
        ])
    }
}

impl SqlEntity for ThoughtEntity {
    fn hydrate(row: Row) -> Result<Self, HydrationError>
    where
        Self: Sized,
    {
        let created_at = DateTime::parse_from_rfc2822(&row.get::<_, String>("created_at"))
            .map_err(|e| {
                HydrationError::InvalidData(format!(
                    "Could not parse DateTime data for field 'created_at'. Error = '{e:?}'."
                ))
            })?;

        Ok(Self {
            thought_id: row.get("thought_id"),
            parent_thought_id: row.get("parent_thought_id"),
            keywords: row.get("keywords"),
            categories: row.get("categories"),
            sources: row.get("sources"),
            created_at: created_at.into(),
            content: row.get("content"),
        })
    }
}

pub struct ThoughtEntitySqlDefinition {
    projection: Projection<ThoughtEntity>,
    source_aliases: SourceAliases,
}

impl SqlDefinition for ThoughtEntitySqlDefinition {
    fn expand(&self, condition: &str) -> String {
        let projection = self.projection.expand(&self.source_aliases);

        format!("select {projection} from thought.thought where {condition}")
    }
}

pub struct ThoughtEntityRepository<'client> {
    provider: Provider<'client, ThoughtEntity>,
}

impl<'client> ThoughtEntityRepository<'client> {
    pub fn new(provider: Provider<'client, ThoughtEntity>) -> Self {
        Self { provider }
    }

    pub async fn get_thought(&self, thought_id: &Uuid) -> StdResult<Option<ThoughtEntity>> {
        let condition = WhereCondition::new("thought_id = $?", params![thought_id]);
        let entity = self
            .provider
            .fetch(condition)
            .await
            .map_err(|e| anyhow!(e))?
            .pop();

        Ok(entity)
    }
}
