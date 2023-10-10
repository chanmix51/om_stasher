use std::sync::Arc;

use crate::thoughts::ThoughtService;

pub struct ServicesContainer {
    thought_service: Arc<dyn ThoughtService>,
}

impl ServicesContainer {
    pub fn new(thought_service: Arc<dyn ThoughtService>) -> Self {
        Self { thought_service }
    }
}
