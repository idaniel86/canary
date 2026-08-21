#![no_std]
#![no_main]

use {defmt_rtt as _, panic_probe as _}; // global logger + panicking-behavior

use bme688;
use defmt::*;
use embassy_executor::Spawner;
use embassy_time::Timer;
use opt3001;

mod hardware;
use embassy_embedded_hal::shared_bus::asynch::i2c::I2cDevice;
use hardware::{Hardware, I2cBus};

struct Delay;

impl embedded_hal_async::delay::DelayNs for Delay {
    async fn delay_ns(&mut self, ns: u32) {
        Timer::after(embassy_time::Duration::from_nanos(ns as u64)).await;
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    // Initialize the hardware peripherals
    let Hardware { i2c_bus } = Hardware::default();

    info!("Hello World!");

    // Spawn sensor tasks
    spawner.spawn(opt3001_task(&i2c_bus).unwrap());
    spawner.spawn(bme688_task(&i2c_bus).unwrap());

    loop {
        Timer::after(embassy_time::Duration::from_secs(1)).await;
        defmt::info!("Heartbeat");
    }
}

#[embassy_executor::task]
async fn opt3001_task(i2c_bus: &'static I2cBus<'static>) {
    let mut sensor =
        opt3001::Opt3001::new(I2cDevice::new(&i2c_bus), opt3001::SlaveAddress::default());
    sensor
        .set_conversion_mode(opt3001::ConversionMode::Continuous)
        .await
        .map_err(|e| error!("Error setting conversion mode: {:?}", e))
        .unwrap();

    loop {
        Timer::after(embassy_time::Duration::from_millis(800)).await;
        if let Ok(lux) = sensor
            .get_result()
            .await
            .map_err(|e| error!("Error reading light intensity: {:?}", e))
        {
            info!("Light Intensity: {} lux", lux);
        }
    }
}

#[embassy_executor::task]
async fn bme688_task(i2c_bus: &'static I2cBus<'static>) {
    let mut sensor = bme688::Bme688::new(
        I2cDevice::new(&i2c_bus),
        bme688::SlaveAddress::default(),
        Delay,
    ).init()
        .await
        .map_err(|e| error!("Error initializing BME688 sensor: {:?}", e))
        .unwrap();
    let config = bme688::sequential::ConfigBuilder::new()
        .with_temperature_os(bme688::Oversampling::X2)
        .with_pressure_os(bme688::Oversampling::X1)
        .with_humidity_os(bme688::Oversampling::X16)
        .with_filter(bme688::Filter::Off)
        .with_heater_profile(&[bme688::sequential::HeaterStep::new(300, 150)], Some(bme688::StandbyTime::Ms1000))
        .build();
    let duration_us = sensor.start_sequential_measurement(&config)
        .await
        .map_err(|e| error!("Error starting BME688 sequential measurement: {:?}", e))
        .unwrap();

    loop {
        Timer::after(embassy_time::Duration::from_micros((duration_us + 1150) as u64)).await;
        if let Ok(measurements) = sensor
            .get_measurements()
            .await
            .map_err(|e| error!("Error reading BME688 measurements: {:?}", e))
        {
            for measurement in measurements {
                info!(
                    "Temp: {} °C, Hum: {} %, Press: {} Pa, Gas: {} Ω",
                    measurement.temperature,
                    measurement.humidity,
                    measurement.pressure,
                    measurement.gas_resistance,
                );    
            }
        }
    }
}
