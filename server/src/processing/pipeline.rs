use crate::domain::SensorReading;
use crate::realtime::EventBus;

/// A processing pipeline that handles incoming sensor readings and publishes them to the event bus.
#[derive(Clone)]
pub struct Pipeline {
    // Add fields for the Pipeline struct here, such as channels or state needed for processing
    pub event_bus: EventBus,
}

impl Pipeline {
    /// Creates a new instance of the Pipeline with the given EventBus.
    ///
    /// # Arguments
    /// * `event_bus` - An instance of EventBus used to publish sensor readings.
    ///
    /// # Returns
    /// * `Pipeline` - A new instance of the Pipeline.
    pub fn new(event_bus: EventBus) -> Self {
        Pipeline { event_bus }
    }

    /// Processes an incoming sensor reading and publishes it to the event bus.
    ///
    /// # Arguments
    /// * `reading` - The sensor reading to be processed.
    pub fn process(&self, reading: SensorReading) {
        self.event_bus.publish(reading);
    }
}
