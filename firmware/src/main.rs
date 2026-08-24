#![no_std]
#![no_main]

use core::sync::atomic::{AtomicI16, AtomicU16, AtomicU32, Ordering};

use {defmt_rtt as _, panic_probe as _}; // global logger + panicking-behavior

use bme688;
use defmt::*;
use embassy_executor::Spawner;
use embassy_time::Timer;
use opt3001;

mod hardware;
use embassy_embedded_hal::shared_bus::asynch::i2c::I2cDevice;
use hardware::{Ethernet, Hardware, I2cBus};

mod proto {
    #![allow(clippy::all)]
    #![allow(nonstandard_style, unused, irrefutable_let_patterns)]
    include!(concat!(env!("OUT_DIR"), "/readings.rs"));
}

#[allow(unused_imports)]
use proto::readings_ as readings;

static PRESSURE: AtomicU32 = AtomicU32::new(99999);
static TEMPERATURE: AtomicI16 = AtomicI16::new(2500);
static HUMIDITY: AtomicU16 = AtomicU16::new(6000);
static LUX: AtomicU16 = AtomicU16::new(0);
static CO2: AtomicU16 = AtomicU16::new(0);
static GAS_RESISTANCE: AtomicU32 = AtomicU32::new(0);

struct Delay;

impl embedded_hal_async::delay::DelayNs for Delay {
    async fn delay_ns(&mut self, ns: u32) {
        Timer::after(embassy_time::Duration::from_nanos(ns as u64)).await;
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    // Initialize the hardware peripherals
    let Hardware {
        i2c_bus,
        net_stack,
        net_runner,
    } = Hardware::default();

    info!("Hello World!");

    // Spawn tasks
    spawner.spawn(opt3001_task(&i2c_bus).unwrap());
    spawner.spawn(bme688_task(&i2c_bus).unwrap());
    spawner.spawn(scd41_task(&i2c_bus).unwrap());
    spawner.spawn(net_task(net_runner).unwrap());

    // Ensure DHCP configuration is up before trying connect
    net_stack.wait_config_up().await;
    info!(
        "Network stack is up. IP address: {}",
        net_stack.config_v4().unwrap().address
    );

    loop {
        Timer::after(embassy_time::Duration::from_secs(1)).await;
        defmt::info!(
            "Temp: {} °C, Hum: {} %, Press: {} Pa, CO2: {} ppm, Gas: {} Ω",
            TEMPERATURE.load(Ordering::Relaxed) as f32 / 100.0,
            HUMIDITY.load(Ordering::Relaxed) as f32 / 100.0,
            PRESSURE.load(Ordering::Relaxed),
            CO2.load(Ordering::Relaxed),
            GAS_RESISTANCE.load(Ordering::Relaxed)
        );
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
            LUX.store(lux as u16, Ordering::Relaxed);
        }
    }
}

#[embassy_executor::task]
async fn bme688_task(i2c_bus: &'static I2cBus<'static>) {
    let mut sensor = bme688::Bme688::new(
        I2cDevice::new(&i2c_bus),
        bme688::SlaveAddress::default(),
        Delay,
    )
    .init()
    .await
    .map_err(|e| error!("Error initializing BME688 sensor: {:?}", e))
    .unwrap();

    loop {
        let ambient_temp = TEMPERATURE.load(Ordering::Relaxed);
        let config = bme688::forced::ConfigBuilder::new()
            .with_temperature_os(bme688::Oversampling::X2)
            .with_pressure_os(bme688::Oversampling::X1)
            .with_humidity_os(bme688::Oversampling::X16)
            .with_filter(bme688::Filter::Off)
            .with_heater_step(300, 150)
            .with_ambient_temperature(ambient_temp / 100)
            .build();

        let duration_us = sensor
            .start_forced_measurement(&config)
            .await
            .map_err(|e| error!("Error starting BME688 forced measurement: {:?}", e))
            .unwrap();

        Timer::after_micros(duration_us as u64 + 150_000).await;
        if let Ok(measurements) = sensor
            .get_measurements()
            .await
            .map_err(|e| error!("Error reading BME688 measurements: {:?}", e))
        {
            if let Some(measurement) = measurements.first() {
                PRESSURE.store(measurement.pressure as u32, Ordering::Relaxed);
                GAS_RESISTANCE.store(measurement.gas_resistance as u32, Ordering::Relaxed);
            }
        }
        Timer::after_secs(1).await;
    }
}

#[embassy_executor::task]
async fn scd41_task(i2c_bus: &'static I2cBus<'static>) {
    let mut sensor = scd4x::Scd4xAsync::new(I2cDevice::new(&i2c_bus), Delay);
    let _ = sensor.stop_periodic_measurement().await;
    sensor
        .reinit()
        .await
        .map_err(|e| error!("Error reinit SCD41: {:?}", e))
        .unwrap();

    sensor
        .start_periodic_measurement()
        .await
        .map_err(|e| error!("Error starting SCD41 periodic measurement: {:?}", e))
        .unwrap();

    loop {
        // Update ambient pressure
        let pressure_hpa = (PRESSURE.load(Ordering::Relaxed) / 100) as u16;
        sensor
            .set_ambient_pressure(pressure_hpa)
            .await
            .map_err(|e| error!("Error setting SCD41 ambient pressure: {:?}", e))
            .unwrap();

        Timer::after(embassy_time::Duration::from_secs(5)).await;
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
                    CO2.store(measurement.co2, Ordering::Relaxed);
                    TEMPERATURE.store((measurement.temperature * 100.0) as i16, Ordering::Relaxed);
                    HUMIDITY.store((measurement.humidity * 100.0) as u16, Ordering::Relaxed);
                }
            }
        }
    }
}

#[embassy_executor::task]
async fn net_task(mut runner: embassy_net::Runner<'static, Ethernet>) -> ! {
    runner.run().await
}
