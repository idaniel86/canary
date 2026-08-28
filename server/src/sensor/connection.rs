use crate::domain;
use crate::processing;
use crate::protocol::framing;
use crate::protocol::proto;

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
        let reading: proto::SensorReading = framing::read_message(&mut socket)
            .await
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        if let Some(temperature) = reading.temperature {
            let sensor_reading = domain::SensorReading {
                timestamp,
                sensor: domain::Sensor::Temperature { value: temperature },
            };
            pipeline.process(sensor_reading);
        }

        if let Some(humidity) = reading.humidity {
            let sensor_reading = domain::SensorReading {
                timestamp,
                sensor: domain::Sensor::Humidity { value: humidity },
            };
            pipeline.process(sensor_reading);
        }

        if let Some(pressure) = reading.pressure {
            let sensor_reading = domain::SensorReading {
                timestamp,
                sensor: domain::Sensor::Pressure { value: pressure },
            };
            pipeline.process(sensor_reading);
        }

        if let Some(co2) = reading.co2 {
            let sensor_reading = domain::SensorReading {
                timestamp,
                sensor: domain::Sensor::CO2 { value: co2 },
            };
            pipeline.process(sensor_reading);
        }

        if let Some(lux) = reading.lux {
            let sensor_reading = domain::SensorReading {
                timestamp,
                sensor: domain::Sensor::Lux { value: lux },
            };
            pipeline.process(sensor_reading);
        }

        if let Some(noise) = reading.noise {
            let sensor_reading = domain::SensorReading {
                timestamp,
                sensor: domain::Sensor::Noise { value: noise },
            };
            pipeline.process(sensor_reading);
        }

        for proto::GasResistance {
            profile,
            resistance,
        } in reading.gas_resistance
        {
            let sensor_reading = domain::SensorReading {
                timestamp,
                sensor: domain::Sensor::GasResistance {
                    profile,
                    resistance,
                },
            };
            pipeline.process(sensor_reading);
        }
    }
}
