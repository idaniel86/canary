mod config;
mod domain;
mod error;
mod processing;
mod protocol;
mod realtime;
mod sensor;
mod web;
mod logging;

pub use error::Error;
use tracing::info;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let config = config::Config::load()?;

    logging::init_logging(&config.logging.level);

    let sensor_listener = tokio::net::TcpListener::bind(config.sensor.address).await?;
    let event_bus = realtime::EventBus::new(config.sensor.capacity);
    let pipeline = processing::Pipeline::new(event_bus);

    let state = web::AppState {
        events: pipeline.event_bus.clone(),
    };
    let app = web::create_router(state);
    let listener = tokio::net::TcpListener::bind(config.web.address).await?;

    info!(
        "Server is running. Sensor listening on {}, Web listening on {}",
        config.sensor.address, config.web.address
    );
    
    let (sensor_result, web_result) = tokio::join!(
        sensor::server::run(sensor_listener, pipeline),
        axum::serve(listener, app),
    );
    sensor_result?;
    web_result?;

    Ok(())
}
