use crate::domain;
use tokio::sync::broadcast;

/// EventBus is a simple event bus that allows broadcasting events to multiple subscribers.
#[derive(Clone)]
pub struct EventBus {
    // The sender is used to broadcast events to all subscribers.
    pub sender: broadcast::Sender<domain::QualityScore>,
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
    /// * `score` - The quality score event to be broadcasted.
    pub fn publish(&self, score: domain::QualityScore) {
        println!("Broadcasting event: {:?}", score);
        let _ = self.sender.send(score);
    }

    /// Subscribes to the event bus, returning a receiver that can be used to receive events.
    ///
    /// # Returns
    /// * `broadcast::Receiver<domain::QualityScore>` - A receiver that can be used to receive events from the event bus.
    pub fn subscribe(&self) -> broadcast::Receiver<domain::QualityScore> {
        self.sender.subscribe()
    }
}
