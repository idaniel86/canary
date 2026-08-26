use askama::Template;
use axum::response::Html;

#[derive(Template)]
#[template(path = "dashboard.html")]
pub struct DashboardTemplate;

/// Handler for the dashboard route
///
/// Returns an HTML response containing the rendered dashboard template.
pub async fn dashboard_handler() -> Html<String> {
    let template = DashboardTemplate;
    Html(template.render().unwrap())
}
