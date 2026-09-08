use crate::Bme688Sensor;
use crate::tasks::ReadingChannel;
use defmt::*;
use embassy_time::{Duration, Timer};

#[embassy_executor::task]
pub async fn bme688_task(
    mut sensor: Bme688Sensor,
    period: Duration,
    _sender: &'static ReadingChannel,
) {
    loop {
        Timer::after(period).await;
        if let Ok(measurements) = sensor
            .get_measurements()
            .await
            .map_err(|e| error!("Error reading BME688 measurements: {:?}", e))
        {
            for _measurement in measurements.iter() {}
        }
    }
}
