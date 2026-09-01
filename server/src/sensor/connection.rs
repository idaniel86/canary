use crate::domain;
use crate::processing;
use crate::protocol::framing;
use crate::protocol::proto;

impl From<proto::QualityScore> for domain::QualityScore {
    fn from(qs: proto::QualityScore) -> Self {
        domain::QualityScore {
            score: qs.score,
            co2: domain::SubScores {
                score: qs.co2.unwrap().score,
                measurement: qs.co2.unwrap().measurement,
            },
            temperature: domain::SubScores {
                score: qs.temperature.unwrap().score,
                measurement: qs.temperature.unwrap().measurement,
            },
            humidity: domain::SubScores {
                score: qs.humidity.unwrap().score,
                measurement: qs.humidity.unwrap().measurement,
            },
            illuminance: domain::SubScores {
                score: qs.illuminance.unwrap().score,
                measurement: qs.illuminance.unwrap().measurement,
            },
            noise: domain::SubScores {
                score: qs.noise.unwrap().score,
                measurement: qs.noise.unwrap().measurement,
            },
        }
    }
}

/// Handles an incoming TCP connection, reading `SensorReading` messages from the socket and processing them.
///
/// # Arguments
/// * `socket` - The TCP stream representing the connection to the sensor.
///
/// # Returns
/// * `Result<(), std::io::Error>` - Returns `Ok(())` on successful processing of the connection,
/// or an `std::io::Error` if an error occurs while reading from the socket or decoding the messages.
pub async fn handle_connection(
    mut socket: tokio::net::TcpStream,
    pipeline: processing::Pipeline,
) -> std::io::Result<()> {
    loop {
        let qs: proto::QualityScore = framing::read_message(&mut socket)
            .await
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        pipeline.process(qs.into());
    }
}
