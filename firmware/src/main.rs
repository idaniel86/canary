#![no_std]
#![no_main]

use {defmt_rtt as _, panic_probe as _}; // global logger + panicking-behavior

use defmt::*;
use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use opt3001;

mod hardware;
use hardware::Hardware;
use embassy_embedded_hal::shared_bus::asynch::i2c::I2cDevice;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    // Initialize the hardware peripherals
    let Hardware { i2c_bus } = Hardware::default();

    info!("Hello World!");

    let mut opt3001_sensor = opt3001::Opt3001::new(I2cDevice::new(&i2c_bus), opt3001::SlaveAddress::default());
    opt3001_sensor
        .set_conversion_mode(opt3001::ConversionMode::Continuous)
        .await
        .map_err(|e| error!("Error setting conversion mode: {:?}", e))
        .unwrap();

    loop {
        Timer::after(Duration::from_millis(800)).await;
        opt3001_sensor
            .get_result()
            .await
            .map(|lux| info!("Lux: {}", lux))
            .unwrap_or_else(|e| error!("Error reading result: {:?}", e));
    }
}
