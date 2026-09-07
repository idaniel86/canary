mod api;
mod handlers;
mod state;

use picoserve::routing::get;
pub use state::AppState;

/// The main application struct for the web interface.
pub struct App<'a> {
    _phantom: core::marker::PhantomData<&'a ()>,
}

impl<'a> App<'a> {
    /// Creates a new instance of the web application.
    pub fn new() -> Self {
        Self {
            _phantom: core::marker::PhantomData,
        }
    }
}

impl<'a> picoserve::AppWithStateBuilder for App<'a> {
    type State = AppState<'a>;
    type PathRouter = impl picoserve::routing::PathRouter<Self::State>;

    fn build_app(self) -> picoserve::Router<Self::PathRouter, Self::State> {
        picoserve::Router::from_service(
            const {
                use picoserve::response::File;

                picoserve::response::Directory {
                    files: &[
                        ("", File::html(include_str!("templates/dashboard.html"))),
                        (
                            "static/dashboard.js",
                            File::javascript(include_str!("static/dashboard.js")),
                        ),
                    ],
                    sub_directories: &[],
                }
            },
        )
        .nest("/api", api::router())
        .route("/events", get(handlers::events::get_events))
    }
}
