use prost::Message;
use tokio::io::AsyncReadExt;
use tokio::sync::broadcast;

mod config;
mod error;
mod readings {
    #![allow(clippy::all)]
    #![allow(nonstandard_style, unused, irrefutable_let_patterns)]
    include!(concat!(env!("OUT_DIR"), "/readings.rs"));
}
pub use error::Error;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let config = config::Config::load()?;

    let tcp_listener = tokio::net::TcpListener::bind(config.server.sensor_address).await?;
    let (sensor_tx, _) = broadcast::channel::<readings::SensorReading>(1024);

    loop {
        let (socket, addr) = tcp_listener.accept().await?;
        println!("New connection from: {}", addr);

        let sensor_tx = sensor_tx.clone();
        tokio::spawn(async move {
            handle_connection(socket, sensor_tx).await.unwrap_or_else(|e| {
                eprintln!("Error handling connection from {}: {}", addr, e);
            });
        });
    }
}

/// Reads a length-delimited `SensorReading` message from the given TCP stream.
/// 
/// # Arguments
/// * `socket` - A mutable reference to a `tokio::net::TcpStream` from which to read the message.
/// 
/// # Returns
/// * `std::io::Result<readings::SensorReading>` - On success, returns the decoded `SensorReading` message. On failure, returns an `std::io::Error.
async fn read_sensor_readings(
    socket: &mut tokio::net::TcpStream,
) -> std::io::Result<readings::SensorReading> {
    let mut buf: Vec<u8> = Vec::new();

    socket.read_buf(&mut buf).await?;
    readings::SensorReading::decode_length_delimited(buf.as_slice())
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
}

/// Handles a single TCP connection, reading `SensorReading` messages in a loop.
/// 
/// # Arguments
/// * `socket` - A mutable `tokio::net::TcpStream` representing the client connection.
/// 
/// # Returns
/// * `std::io::Result<()>` - Returns `Ok(())` on successful handling of the connection, or an `std::io::Error` if an error occurs during reading or decoding messages.
async fn handle_connection(mut socket: tokio::net::TcpStream, sensor_tx: broadcast::Sender<readings::SensorReading>) -> std::io::Result<()> {
    loop {
        // Read a length-delimited `SensorReading` message from the socket.
        let reading = read_sensor_readings(&mut socket).await?;

        // Send to all subscribed WebSocket clients.
        let _ = sensor_tx.send(reading);
    }
}
