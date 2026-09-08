use crate::Scd41Sensor;
use crate::quality::Reading;
use crate::tasks::ReadingChannel;
use defmt::*;
use embassy_time::{Duration, Timer};

#[embassy_executor::task]
pub async fn scd41_task(
    mut sensor: Scd41Sensor,
    period: Duration,
    sender: &'static ReadingChannel,
) {
    let sender = sender.sender();

    loop {
        Timer::after(period).await;
        if let Ok(is_data_ready) = sensor
            .data_ready_status()
            .await
            .map_err(|e| error!("Error reading SCD41 data ready status: {:?}", e))
        {
            if is_data_ready {
                if let Ok(measurement) = sensor
                    .measurement()
                    .await
                    .map_err(|e| error!("Error reading SCD41 measurement: {:?}", e))
                {
                    // Send the readings to the channel
                    sender.send(Reading::Co2(measurement.co2 as f32)).await;
                    sender
                        .send(Reading::Temperature(measurement.temperature))
                        .await;
                    sender.send(Reading::Humidity(measurement.humidity)).await;
                }
            }
        }
    }
}
