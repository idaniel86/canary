mod api;
mod state;

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
        picoserve::Router::new().nest("/api", api::router())
    }
}
