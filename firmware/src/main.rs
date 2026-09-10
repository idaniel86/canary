#![no_std]
#![no_main]
#![feature(impl_trait_in_assoc_type)]

use crate::quality::{AnyQualityModel, NonLinearQualityModel, Subscores, WeightedQualityModel};

use {defmt_rtt as _, panic_probe as _}; // global logger + panicking-behavior

use bme688;
use defmt::*;
use embassy_embedded_hal::shared_bus::asynch::i2c::I2cDevice;
use embassy_executor::Spawner;
use embassy_sync::{blocking_mutex::raw::NoopRawMutex, mutex::Mutex};
use embassy_time::{Duration, Timer};
use hardware::Hardware;
use opt3001;
use picoserve::AppWithStateBuilder;

mod filters;
mod hardware;
mod ics43434;
mod quality;
mod storage;
use storage::Storage;
mod tasks;
mod web;

pub struct Delay;

impl embedded_hal_async::delay::DelayNs for Delay {
    async fn delay_ns(&mut self, ns: u32) {
        Timer::after(embassy_time::Duration::from_nanos(ns as u64)).await;
    }
}

pub type Opt3001Sensor = opt3001::Opt3001<
    I2cDevice<
        'static,
        NoopRawMutex,
        embassy_stm32::i2c::I2c<'static, embassy_stm32::mode::Async, embassy_stm32::i2c::Master>,
    >,
>;
pub type Scd41Sensor = scd4x::Scd4xAsync<
    I2cDevice<
        'static,
        NoopRawMutex,
        embassy_stm32::i2c::I2c<'static, embassy_stm32::mode::Async, embassy_stm32::i2c::Master>,
    >,
    Delay,
>;
pub type Bme688Sensor = bme688::Bme688<
    I2cDevice<
        'static,
        NoopRawMutex,
        embassy_stm32::i2c::I2c<'static, embassy_stm32::mode::Async, embassy_stm32::i2c::Master>,
    >,
    Delay,
    bme688::Init,
>;

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    // Initialize the hardware peripherals
    let Hardware {
        i2c_bus,
        net_stack,
        net_runner,
        mut mic_sai,
        flash,
    } = Hardware::default();

    info!("Hello World!");

    let mut storage = Storage::new(flash);
    let score_config = storage
        .get_score_config()
        .await
        .map_err(|e| error!("Failed to get score config: {:?}", e))
        .unwrap_or_default()
        .unwrap_or_default();

    static SHARED_STATE: static_cell::StaticCell<tasks::SharedState> =
        static_cell::StaticCell::new();
    let shared_state = SHARED_STATE.init(tasks::SharedState {
        quality: Mutex::new(tasks::Quality {
            subscores: Subscores::new(&score_config),
            model: AnyQualityModel::NonLinear(NonLinearQualityModel {}),
            score_config,
        }),
        storage_signal: embassy_sync::signal::Signal::new(),
    });

    static READING_CHANNEL: static_cell::StaticCell<tasks::ReadingChannel> =
        static_cell::StaticCell::new();
    let reading_channel = READING_CHANNEL.init(embassy_sync::channel::Channel::new());

    static AUDIO_CHANNEL: static_cell::StaticCell<tasks::AudioChannel> =
        static_cell::StaticCell::new();
    let audio_channel = AUDIO_CHANNEL.init(embassy_sync::pubsub::PubSubChannel::new());

    let mut opt3001_sensor: Opt3001Sensor =
        opt3001::Opt3001::new(I2cDevice::new(&i2c_bus), opt3001::SlaveAddress::default());
    opt3001_sensor
        .set_conversion_mode(opt3001::ConversionMode::Continuous)
        .await
        .map_err(|e| error!("Error setting conversion mode: {:?}", e))
        .unwrap();

    let mut scd41_sensor = scd4x::Scd4xAsync::new(I2cDevice::new(&i2c_bus), Delay);
    let _ = scd41_sensor.stop_periodic_measurement().await;
    scd41_sensor
        .reinit()
        .await
        .map_err(|e| error!("Error reinit SCD41: {:?}", e))
        .unwrap();

    scd41_sensor
        .start_periodic_measurement()
        .await
        .map_err(|e| error!("Error starting SCD41 periodic measurement: {:?}", e))
        .unwrap();

    let mut bme688_sensor = bme688::Bme688::new(
        I2cDevice::new(&i2c_bus),
        bme688::SlaveAddress::default(),
        Delay,
    )
    .init()
    .await
    .map_err(|e| error!("Error initializing BME688 sensor: {:?}", e))
    .unwrap();

    let config = bme688::sequential::ConfigBuilder::new()
        .with_temperature_os(bme688::Oversampling::X2)
        .with_pressure_os(bme688::Oversampling::X1)
        .with_humidity_os(bme688::Oversampling::X16)
        .with_filter(bme688::Filter::Off)
        .with_heater_profile(
            &[
                bme688::sequential::HeaterStep::new(200, 280),
                bme688::sequential::HeaterStep::new(225, 280),
                bme688::sequential::HeaterStep::new(250, 280),
                bme688::sequential::HeaterStep::new(275, 280),
                bme688::sequential::HeaterStep::new(300, 280),
                bme688::sequential::HeaterStep::new(325, 280),
                bme688::sequential::HeaterStep::new(350, 280),
                bme688::sequential::HeaterStep::new(375, 280),
                bme688::sequential::HeaterStep::new(400, 280),
                bme688::sequential::HeaterStep::new(350, 280),
            ],
            Some(bme688::StandbyTime::Ms1000),
        )
        .build();

    let duration_us = bme688_sensor
        .start_sequential_measurement(&config)
        .await
        .map_err(|e| error!("Error starting BME688 sequential measurement: {:?}", e))
        .unwrap();

    let duration = (embassy_time::Duration::from_micros(duration_us as u64)
        + embassy_time::Duration::from_millis(280))
        * 3;

    mic_sai
        .start()
        .map_err(|e| error!("Error starting MIC SAI: {:?}", e))
        .unwrap();

    spawner.spawn(tasks::sai_task(mic_sai, audio_channel).unwrap());
    spawner.spawn(
        tasks::opt3001_task(opt3001_sensor, Duration::from_millis(800), reading_channel).unwrap(),
    );
    spawner
        .spawn(tasks::scd41_task(scd41_sensor, Duration::from_secs(5), reading_channel).unwrap());
    spawner.spawn(
        tasks::ics43434_task(Duration::from_secs(1), audio_channel, reading_channel).unwrap(),
    );
    spawner.spawn(tasks::bme688_task(bme688_sensor, duration, reading_channel).unwrap());
    spawner.spawn(tasks::aggregator_task(shared_state, reading_channel).unwrap());
    spawner.spawn(tasks::net_task(net_runner).unwrap());
    spawner.spawn(tasks::storage_task(storage, shared_state).unwrap());

    // Ensure DHCP configuration is up before trying connect
    net_stack.wait_config_up().await;
    info!(
        "Network stack is up. IP address: {}",
        net_stack.config_v4().unwrap().address
    );

    let app = picoserve::make_static!(picoserve::AppRouter<web::App>, web::App::new().build_app());
    let app_state = picoserve::make_static!(
        web::AppState,
        web::AppState::new(&shared_state.quality, &shared_state.storage_signal),
    );

    for task_id in 0..tasks::WEB_TASK_POOL_SIZE {
        spawner.spawn(tasks::web_task(task_id, net_stack, app, app_state).unwrap());
    }

    loop {
        info!("Heartbeat...");
        Timer::after(embassy_time::Duration::from_secs(60)).await;
    }
}
