mod score;

use crate::web::AppState;

/// Returns a router for handling API endpoints.
pub fn router<'a>()
-> picoserve::Router<impl picoserve::routing::PathRouter<AppState<'a>>, AppState<'a>> {
    picoserve::Router::new().nest("/score", score::router())
}
