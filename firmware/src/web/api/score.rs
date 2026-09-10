use crate::{
    quality::ScoreConfig,
    web::{AppState, state::QualityState},
};
use picoserve::{
    Router,
    extract::{Json, State},
    response::{IntoResponse, StatusCode},
    routing::{PathRouter, get},
};

/// Returns a router for handling quality score configuration endpoints.
pub fn router<'a>() -> Router<impl PathRouter<AppState<'a>>, AppState<'a>> {
    Router::new()
        .route("/config", get(get_score_config).post(set_score_config))
        .route("/current", get(get_current_score))
}

/// Handles the GET request for retrieving the current quality score configuration.
async fn get_score_config(
    State(QualityState(state)): State<QualityState<'_>>,
) -> impl IntoResponse {
    let quality = state.lock().await;
    Json(quality.score_config.clone())
}
/// Handles the POST request for updating the quality score configuration.
async fn set_score_config(
    State(AppState {
        quality,
        storage_signal,
    }): State<AppState<'_>>,
    Json(config): Json<ScoreConfig>,
) -> impl IntoResponse {
    let mut quality = quality.lock().await;
    quality.score_config = config;
    // Signal that the quality score configuration has been updated
    storage_signal.signal(());
    StatusCode::NO_CONTENT
}

/// Handles the GET request for retrieving the current quality score.
async fn get_current_score(
    State(QualityState(state)): State<QualityState<'_>>,
) -> impl IntoResponse {
    let quality = state.lock().await;
    Json(
        quality
            .subscores
            .snapshot(&quality.model, &quality.score_config),
    )
}
