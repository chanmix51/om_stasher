use async_trait::async_trait;

use crate::StdResult;

#[async_trait]
pub trait Runtime {
    async fn run(&self) -> StdResult<()>;
}
