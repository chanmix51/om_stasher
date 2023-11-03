use std::sync::Arc;

use async_trait::async_trait;
use tokio::{
    sync::{broadcast::Receiver, Mutex},
    task::yield_now,
};

use crate::{EventMessage, StdResult};

/// Services are associated to a runtime that reacts to events.
#[async_trait]
pub trait ServiceRuntime {
    /// How the runtime reacts to external events
    async fn process_event(&self, event: EventMessage) -> StdResult<()>;

    /// Return the service identifier of the current service runtime.
    /// This allows to filter incoming events, ignoring the events sent by our own service.
    fn get_service_id(&self) -> u8;
}

pub struct Runtime<T>
where
    T: ServiceRuntime,
{
    service_runtime: Arc<T>,
    broadcast_receiver: Arc<Mutex<Receiver<EventMessage>>>,
}

impl<T> Runtime<T>
where
    T: ServiceRuntime,
{
    pub fn new(
        service_runtime: Arc<T>,
        broadcast_receiver: Arc<Mutex<Receiver<EventMessage>>>,
    ) -> Self {
        Self {
            service_runtime,
            broadcast_receiver,
        }
    }

    pub async fn run(&self) -> StdResult<()> {
        loop {
            match self.broadcast_receiver.lock().await.recv().await {
                Err(_e) => return Ok(()),
                Ok(event) if event.origin == self.service_runtime.get_service_id() => continue,
                Ok(event) => self.service_runtime.process_event(event).await?,
            }

            // TODO: check the performances impact of this ↓
            // I expect it should give it back to Tokio scheduler before polling again the
            // receiver. This aims at not starving other tasks if there are a lot of incoming
            // events.
            yield_now().await;
        }
    }
}
