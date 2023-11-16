use std::sync::Arc;

use anyhow::anyhow;
use async_trait::async_trait;
use log::debug;
use tokio::sync::broadcast::{
    channel as broadcast_channel, Receiver as BroadcastReceiver, Sender as BroadcastSender,
};
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};
use tokio::sync::Mutex;

use crate::{ServiceRuntime, StdResult};

/// Size of subscribers' channels buffers.
/// All registered subscribers will allocate a channel with this buffer length.
const SUBSCRIBER_CHANNEL_SIZE: usize = 5;

/// State modification advertised by a message
/// Each variant data must be a public identifier of the entity it refers to.
#[derive(Debug, Clone, PartialEq)]
pub enum StateModification {
    Creation(String),
    Update(String),
    Delete(String),
}

/// Message sent from services to advertise state changes.
#[derive(Debug, Clone, PartialEq)]
pub struct EventMessage {
    /// Service the message originates from
    /// By convention, origin shall never be 0.
    pub origin: u8,

    /// The name of the entity the message refers to
    pub subject: String,

    /// The type of state modification with the link to the according data.
    pub action: StateModification,
}

impl EventMessage {
    /// Create a new event message
    pub fn new(origin: u8, subject: &str, action: StateModification) -> Self {
        Self {
            origin,
            subject: subject.to_string(),
            action,
        }
    }
}

/// Publisher/Subscriber dispatcher
pub struct EventDispatcher {
    receiver: Mutex<UnboundedReceiver<EventMessage>>,
    sender: UnboundedSender<EventMessage>,
    broadcast_sender: BroadcastSender<EventMessage>,
}

impl Default for EventDispatcher {
    fn default() -> Self {
        let (sender, receiver) = unbounded_channel::<EventMessage>();
        let (broadcast_sender, _broadcast_receiver) =
            broadcast_channel::<EventMessage>(SUBSCRIBER_CHANNEL_SIZE);

        Self {
            receiver: Mutex::new(receiver),
            sender,
            broadcast_sender,
        }
    }
}

impl EventDispatcher {
    pub fn subscribe(
        &self,
    ) -> (
        UnboundedSender<EventMessage>,
        BroadcastReceiver<EventMessage>,
    ) {
        let sender = self.sender.clone();
        let receiver = self.broadcast_sender.subscribe();

        (sender, receiver)
    }

    pub async fn cycle(&self) -> StdResult<()> {
        match self.receiver.lock().await.recv().await {
            Some(event) => self
                .broadcast_sender
                .send(event)
                .map(|_| ())
                .map_err(|e| anyhow!(e).context("Could not broadcast event")),
            None => Err(anyhow!("No more senders to listen to.")),
        }
    }
}

/// This structure is a debugging service to log events broadcasted by the event dispatcher.
#[derive(Debug, Default)]
pub struct LoggerServiceRuntime;

#[async_trait]
impl ServiceRuntime for LoggerServiceRuntime {
    // There is no service associated with this listener hence there is no way it could be notified
    // from its own events. 0 is a safe value here.
    fn get_service_id(&self) -> u8 {
        0
    }

    async fn process_event(&self, event: EventMessage) -> StdResult<()> {
        debug!("📨→ {event:?}");

        Ok(())
    }
}

/// Event dispatcher loop
///
/// This is run alongside all runtimes
pub struct EventDispatcherLoop {
    dispatcher: Arc<EventDispatcher>,
}

impl EventDispatcherLoop {
    pub fn new(dispatcher: Arc<EventDispatcher>) -> Self {
        Self { dispatcher }
    }

    /// Loop
    pub async fn tickle(&self) -> StdResult<()> {
        loop {
            match self.dispatcher.cycle().await {
                Ok(_) => (),
                Err(e) => {
                    return Err(anyhow!(e));
                }
            };
        }
    }
}

#[cfg(test)]
mod tests {
    use tokio::time::{sleep, Duration};
    use tokio::{select, sync::broadcast::error::TryRecvError};

    use crate::StdResult;

    use super::*;

    #[tokio::test]
    async fn subscribe_simple() -> StdResult<()> {
        let dispatcher = EventDispatcher::default();
        let (sender, mut receiver) = dispatcher.subscribe();
        let handler = tokio::spawn(async move {
            loop {
                dispatcher.cycle().await.unwrap();
            }
        });

        // no message sent, channel must be empty
        assert_eq!(Err(TryRecvError::Empty), receiver.try_recv());

        let message = EventMessage::new(
            0,
            "subject",
            StateModification::Creation("Whatever".to_string()),
        );
        sender.send(message.clone()).unwrap();

        let result = select! {
            message = receiver.recv() => Ok(message),
            _ = sleep(Duration::from_millis(10)) => Err(anyhow!("message RECV timeout"))
        }
        .unwrap();

        // A message has been sent, channel broadcasts the message
        assert_eq!(Ok(message), result);

        // Once delivered, the channel is empty
        assert_eq!(Err(TryRecvError::Empty), receiver.try_recv());

        handler.abort();

        Ok(())
    }

    #[tokio::test]
    async fn two_subscribers() -> StdResult<()> {
        let dispatcher = EventDispatcher::default();
        let (sender1, mut receiver1) = dispatcher.subscribe();
        let (_sender2, mut receiver2) = dispatcher.subscribe();
        let handler = tokio::spawn(async move {
            loop {
                dispatcher.cycle().await.unwrap();
            }
        });

        let message = EventMessage::new(
            0,
            "whatever",
            StateModification::Delete("whatever".to_string()),
        );

        // Only sender 1 sends a message
        sender1.send(message.clone()).unwrap();
        let result = select! {
            message = receiver2.recv() => Ok(message),
            _ = sleep(Duration::from_millis(10)) => Err(anyhow!("receiver2 message RECV timeout"))
        }
        .unwrap();

        // A message has been sent, channel broadcasts the message
        assert_eq!(Ok(message.clone()), result);
        assert_eq!(Ok(message), receiver1.recv().await);

        handler.abort();

        Ok(())
    }
}
