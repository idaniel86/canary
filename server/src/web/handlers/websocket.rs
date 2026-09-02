use crate::web::state::AppState;
use axum::{
    extract::{
        State,
        ws::{Message, WebSocket, WebSocketUpgrade},
    },
    response::IntoResponse,
};
use serde_json;
use tracing::{info, error};

/// Handles WebSocket connections and sends sensor readings to connected clients.
///
/// # Arguments
/// * `ws` - The WebSocket upgrade request
/// * `state` - The application state containing the event bus
///
/// Returns an `IntoResponse` that upgrades the connection to a WebSocket.
pub async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

/// Handles the WebSocket connection by subscribing to the event bus and sending sensor readings to the client.
///
/// # Arguments
/// * `socket` - The WebSocket connection to the client
/// * `state` - The application state containing the event bus
async fn handle_socket(mut socket: WebSocket, state: AppState) {
    info!("New WebSocket connection established.");

    let mut rx = state.events.subscribe();

    while let Ok(reading) = rx.recv().await {
        let json = match serde_json::to_string(&reading) {
            Ok(json) => json,
            Err(e) => {
                error!(error = %e, "Failed to serialize sensor reading");
                continue;
            }
        };

        if let Err(e) = socket.send(Message::Text(json.into())).await {
            error!(error = %e, "Failed to send message over WebSocket");
            break;
        }
    }
}
