use crate::realtime;

/// Represents the application state shared across different parts of the web server.
#[derive(Clone)]
pub struct AppState {
    pub events: realtime::EventBus,
}
