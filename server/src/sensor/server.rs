use super::connection::handle_connection;
use crate::processing::Pipeline;
use tokio::net::TcpListener;

/// Starts the sensor server, accepting incoming TCP connections and processing sensor readings.
///
/// # Arguments
/// * `listener` - A `TcpListener` that listens for incoming TCP connections
/// * `pipeline` - A `Pipeline` instance that processes incoming sensor readings
///
/// # Returns
/// * `Result<(), std::io::Error>` - Returns `Ok(())` on successful processing of the connections, or an `std::io::Error` if an error occurs while accepting connections or processing readings.
pub async fn run(listener: TcpListener, pipeline: Pipeline) -> Result<(), std::io::Error> {
    loop {
        let (socket, addr) = listener.accept().await?;
        println!("New connection from: {}", addr);

        // Handle the connection in a separate task
        let pipeline = pipeline.clone();
        tokio::spawn(async move {
            if let Err(e) = handle_connection(socket, pipeline).await {
                eprintln!("Error handling connection from {}: {}", addr, e);
            }
        });
    }
}
