mod config;
mod domain;
mod error;
mod processing;
mod protocol;
mod realtime;
mod sensor;
mod web;

pub use error::Error;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let config = config::Config::load()?;

    let sensor_listener = tokio::net::TcpListener::bind(config.sensor.address).await?;
    let event_bus = realtime::EventBus::new(config.sensor.capacity);
    let pipeline = processing::Pipeline::new(event_bus);

    let state = web::AppState {
        events: pipeline.event_bus.clone(),
    };
    let app = web::create_router(state);
    let listener = tokio::net::TcpListener::bind(config.web.address).await?;

    let (sensor_result, web_result) = tokio::join!(
        sensor::server::run(sensor_listener, pipeline),
        axum::serve(listener, app),
    );
    sensor_result?;
    web_result?;

    Ok(())
}
