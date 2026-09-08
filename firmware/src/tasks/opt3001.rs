use crate::Opt3001Sensor;
use crate::quality::Reading;
use crate::tasks::ReadingChannel;
use defmt::*;
use embassy_time::{Duration, Timer};

#[embassy_executor::task]
pub async fn opt3001_task(
    mut sensor: Opt3001Sensor,
    period: Duration,
    sender: &'static ReadingChannel,
) {
    let sender = sender.sender();

    loop {
        Timer::after(period).await;
        if let Ok(status) = sensor.get_status().await {
            if status.is_conversion_ready {
                if let Ok(illuminance) = sensor
                    .get_result()
                    .await
                    .map_err(|e| error!("Error reading light intensity: {:?}", e))
                {
                    // Send the illuminance reading to the channel
                    sender.send(Reading::Illuminance(illuminance as f32)).await;
                }
            }
        }
    }
}
