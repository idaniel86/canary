mod config;
mod domain;
mod error;
mod processing;
mod protocol;
mod realtime;
mod sensor;

pub use error::Error;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let config = config::Config::load()?;

    let sensor_listener = tokio::net::TcpListener::bind(config.sensor.address).await?;
    let event_bus = realtime::EventBus::new(config.sensor.capacity);
    let pipeline = processing::Pipeline::new(event_bus);

    sensor::server::run(sensor_listener, pipeline).await?;

    Ok(())
}
