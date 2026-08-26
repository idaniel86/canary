use crate::web::handlers;
use crate::web::state::AppState;
use axum::{Router, routing::get};

/// Creates and returns an Axum router configured with the necessary routes and state.
///
/// # Arguments
/// * `state` - An instance of `AppState` containing shared application state
///
/// # Returns
/// * `Router` - An Axum router with the configured routes and state
pub fn create_router(state: AppState) -> axum::Router {
    Router::new()
        .route("/ws", get(handlers::websocket::websocket_handler))
        .with_state(state)
}
