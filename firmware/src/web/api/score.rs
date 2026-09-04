use crate::{
    quality::QualityScoreConfig,
    web::{
        AppState,
        state::{QualityScoreConfigState, QualityScoreState},
    },
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
    State(QualityScoreConfigState(state)): State<QualityScoreConfigState<'_>>,
) -> impl IntoResponse {
    let config = state.lock().await;
    Json(config.clone())
}
/// Handles the POST request for updating the quality score configuration.
async fn set_score_config(
    State(QualityScoreConfigState(state)): State<QualityScoreConfigState<'_>>,
    Json(new_config): Json<QualityScoreConfig>,
) -> impl IntoResponse {
    let mut config = state.lock().await;
    *config = new_config;
    StatusCode::NO_CONTENT
}

/// Handles the GET request for retrieving the current quality score.
async fn get_current_score(
    State(QualityScoreState(state)): State<QualityScoreState<'_>>,
) -> impl IntoResponse {
    let config = state.lock().await;
    Json(config.clone())
}
