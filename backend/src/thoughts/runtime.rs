use std::sync::Arc;

use async_trait::async_trait;

use crate::{EventMessage, ServiceRuntime, ServicesContainer, StdResult};

pub struct ThoughtServiceRuntime {
    services_container: Arc<ServicesContainer>,
}

impl ThoughtServiceRuntime {
    pub fn new(services_container: Arc<ServicesContainer>) -> Self {
        Self { services_container }
    }
}

#[async_trait]
impl ServiceRuntime for ThoughtServiceRuntime {
    fn get_service_id(&self) -> u8 {
        1
    }

    async fn process_event(&self, event: EventMessage) -> StdResult<()> {
        Ok(())
    }
}
