use crate::domain;
use tokio::sync::broadcast;

/// EventBus is a simple event bus that allows broadcasting events to multiple subscribers.
#[derive(Clone)]
pub struct EventBus {
    // The sender is used to broadcast events to all subscribers.
    pub sender: broadcast::Sender<domain::SensorReading>,
}

impl EventBus {
    /// Creates a new EventBus with the specified capacity.
    /// The capacity determines how many events can be buffered before subscribers start missing events.
    ///
    /// # Arguments
    /// * `capacity` - The maximum number of events that can be buffered.
    ///
    /// # Returns
    /// * `EventBus` - A new instance of the EventBus.
    pub fn new(capacity: usize) -> Self {
        let (sender, _) = broadcast::channel(capacity);
        EventBus { sender }
    }

    /// Publishes a new event to all subscribers.
    ///
    /// # Arguments
    /// * `reading` - The sensor reading event to be broadcasted.
    pub fn publish(&self, reading: domain::SensorReading) {
        println!("Broadcasting event: {:?}", reading);
        let _ = self.sender.send(reading);
    }

    /// Subscribes to the event bus, returning a receiver that can be used to receive events.
    ///
    /// # Returns
    /// * `broadcast::Receiver<domain::SensorReading>` - A receiver that can be used to receive events from the event bus.
    pub fn subscribe(&self) -> broadcast::Receiver<domain::SensorReading> {
        self.sender.subscribe()
    }
}
