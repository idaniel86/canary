#![no_std]
#![no_main]
#![feature(impl_trait_in_assoc_type)]

use {defmt_rtt as _, panic_probe as _}; // global logger + panicking-behavior

use bme688;
use defmt::*;
use embassy_executor::Spawner;
use embassy_sync::{
    blocking_mutex::raw::NoopRawMutex,
    mutex::Mutex,
    signal,
};
use embassy_time::Timer;
use opt3001;

mod hardware;
use embassy_embedded_hal::shared_bus::asynch::i2c::I2cDevice;
use hardware::{Ethernet, Hardware, I2cBus};
mod filters;
mod ics43434;
mod quality;
mod storage;
use storage::Storage;
mod web;

use picoserve::AppWithStateBuilder;

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
        mic_sai,
        flash,
    } = Hardware::default();

    info!("Hello World!");

    let mut storage = Storage::new(flash);
    let score_config = storage
        .get_score_config()
        .await
        .map_err(|e| error!("Failed to get score config: {:?}", e))
        .unwrap()
        .unwrap_or_default();

    static STORAGE_SIGNAL: static_cell::StaticCell<embassy_sync::signal::Signal<NoopRawMutex, ()>> =
        static_cell::StaticCell::new();
    let storage_signal = STORAGE_SIGNAL.init(embassy_sync::signal::Signal::new());

    static QUALITY_SCORE: static_cell::StaticCell<Mutex<NoopRawMutex, quality::QualityScore>> =
        static_cell::StaticCell::new();
    let quality_score = QUALITY_SCORE.init(Mutex::new(quality::QualityScore::new()));

    static QUALITY_SCORE_CONFIG: static_cell::StaticCell<
        Mutex<NoopRawMutex, quality::QualityScoreConfig>,
    > = static_cell::StaticCell::new();
    let quality_score_config = QUALITY_SCORE_CONFIG.init(Mutex::new(score_config));

    // Spawn tasks
    spawner.spawn(opt3001_task(&i2c_bus, quality_score, quality_score_config).unwrap());
    spawner.spawn(bme688_task(&i2c_bus).unwrap());
    spawner.spawn(scd41_task(&i2c_bus, quality_score, quality_score_config).unwrap());
    spawner.spawn(net_task(net_runner).unwrap());
    spawner.spawn(ics_43434_task(mic_sai, quality_score, quality_score_config).unwrap());
    spawner.spawn(storage_task(storage, storage_signal, quality_score_config).unwrap());

    // Ensure DHCP configuration is up before trying connect
    net_stack.wait_config_up().await;
    info!(
        "Network stack is up. IP address: {}",
        net_stack.config_v4().unwrap().address
    );

    let app = picoserve::make_static!(picoserve::AppRouter<web::App>, web::App::new().build_app());
    let app_state = picoserve::make_static!(
        web::AppState,
        web::AppState::new(quality_score, quality_score_config, storage_signal)
    );

    for task_id in 0..WEB_TASK_POOL_SIZE {
        spawner.spawn(web_task(task_id, net_stack, app, app_state).unwrap());
    }

    loop {
        info!("Heartbeat...");
        Timer::after(embassy_time::Duration::from_secs(60)).await;
    }
}

#[embassy_executor::task]
async fn opt3001_task(
    i2c_bus: &'static I2cBus<'static>,
    quality_score: &'static Mutex<NoopRawMutex, quality::QualityScore>,
    quality_score_config: &'static Mutex<NoopRawMutex, quality::QualityScoreConfig>,
) {
    let quality_score_config_lock = quality_score_config.lock().await;
    let mut illuminance_filter = filters::LowPassFilter::new(
        quality_score_config_lock.illuminance.filter_tau_seconds,
        None,
    );
    drop(quality_score_config_lock);

    let mut sensor =
        opt3001::Opt3001::new(I2cDevice::new(&i2c_bus), opt3001::SlaveAddress::default());
    sensor
        .set_conversion_mode(opt3001::ConversionMode::Continuous)
        .await
        .map_err(|e| error!("Error setting conversion mode: {:?}", e))
        .unwrap();

    loop {
        Timer::after(embassy_time::Duration::from_millis(800)).await;
        if let Ok(status) = sensor.get_status().await {
            if status.is_conversion_ready {
                if let Ok(illuminance) = sensor
                    .get_result()
                    .await
                    .map_err(|e| error!("Error reading light intensity: {:?}", e))
                {
                    let illuminance = illuminance_filter.process(illuminance as f32);
                    quality_score
                        .lock()
                        .await
                        .update_illuminance(illuminance, &*quality_score_config.lock().await);
                }
            }
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

    let duration_us = sensor
        .start_sequential_measurement(&config)
        .await
        .map_err(|e| error!("Error starting BME688 sequential measurement: {:?}", e))
        .unwrap();

    let duration = (embassy_time::Duration::from_micros(duration_us as u64)
        + embassy_time::Duration::from_millis(280))
        * 3;

    loop {
        Timer::after(duration).await;

        if let Ok(measurements) = sensor
            .get_measurements()
            .await
            .map_err(|e| error!("Error reading BME688 measurements: {:?}", e))
        {
            for _measurement in measurements.iter() {}
        }
    }
}

#[embassy_executor::task]
async fn scd41_task(
    i2c_bus: &'static I2cBus<'static>,
    quality_score: &'static Mutex<NoopRawMutex, quality::QualityScore>,
    quality_score_config: &'static Mutex<NoopRawMutex, quality::QualityScoreConfig>,
) {
    let quality_score_config_lock = quality_score_config.lock().await;
    let mut co2_filter =
        filters::LowPassFilter::new(quality_score_config_lock.co2.filter_tau_seconds, None);
    let mut temperature_filter = filters::LowPassFilter::new(
        quality_score_config_lock.temperature.filter_tau_seconds,
        None,
    );
    let mut humidity_filter =
        filters::LowPassFilter::new(quality_score_config_lock.humidity.filter_tau_seconds, None);
    drop(quality_score_config_lock);

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
                    let co2_filtered = co2_filter.process(measurement.co2 as f32);
                    let temperature_filtered = temperature_filter.process(measurement.temperature);
                    let humidity_filtered = humidity_filter.process(measurement.humidity);
                    let mut lock = quality_score.lock().await;
                    let config_lock = quality_score_config.lock().await;
                    lock.update_co2(co2_filtered, &config_lock);
                    lock.update_temperature(temperature_filtered, &config_lock);
                    lock.update_humidity(humidity_filtered, &config_lock);
                }
            }
        }
    }
}

#[embassy_executor::task]
async fn net_task(mut runner: embassy_net::Runner<'static, Ethernet>) -> ! {
    runner.run().await
}

#[embassy_executor::task]
async fn ics_43434_task(
    mut mic_sai: embassy_stm32::sai::Sai<'static, embassy_stm32::peripherals::SAI1, u32>,
    quality_score: &'static Mutex<NoopRawMutex, quality::QualityScore>,
    quality_score_config: &'static Mutex<NoopRawMutex, quality::QualityScoreConfig>,
) {
    let quality_score_config_lock = quality_score_config.lock().await;
    let mut spl_filter =
        filters::LowPassFilter::new(quality_score_config_lock.noise.filter_tau_seconds, None);
    drop(quality_score_config_lock);

    let mut ics_43434 = ics43434::Ics43434::new();
    let mut raw_audio_frame = [1u32; 1024]; // Buffer to hold raw audio samples

    const SAMPLE_RATE: u32 = 48_000;

    let mut sample_count = 0;

    mic_sai.start().unwrap_or_else(|e| {
        error!("Failed to start SAI interface: {:?}", e);
    });

    loop {
        if let Err(e) = mic_sai.read(&mut raw_audio_frame).await {
            error!("Error reading from ICS-43434 microphone: {:?}", e);
            continue;
        }

        for &raw_sample in raw_audio_frame.iter() {
            ics_43434.process(raw_sample);

            sample_count += 1;
            if sample_count >= SAMPLE_RATE {
                sample_count = 0;
                let spl = ics_43434.get_spl();
                let spl_filtered = spl_filter.process(spl);
                quality_score
                    .lock()
                    .await
                    .update_noise(spl_filtered, &*quality_score_config.lock().await);
            }
        }
    }
}

static CONFIG: picoserve::Config = picoserve::Config::const_default().keep_connection_alive();

const WEB_TASK_POOL_SIZE: usize = 1;

#[embassy_executor::task(pool_size = WEB_TASK_POOL_SIZE)]
async fn web_task(
    task_id: usize,
    stack: embassy_net::Stack<'static>,
    app: &'static picoserve::AppRouter<web::App<'static>>,
    state: &'static web::AppState<'static>,
) {
    let port = 80;
    let mut tcp_rx_buffer = [0; 1024];
    let mut tcp_tx_buffer = [0; 1024];
    let mut http_buffer = [0; 4096];

    picoserve::Server::new(&app.shared().with_state(state), &CONFIG, &mut http_buffer)
        .listen_and_serve(task_id, stack, port, &mut tcp_rx_buffer, &mut tcp_tx_buffer)
        .await
        .into_never()
}

#[embassy_executor::task]
async fn storage_task(
    mut storage: Storage<'static>,
    signal: &'static signal::Signal<NoopRawMutex, ()>,
    quality_score_config: &'static Mutex<NoopRawMutex, quality::QualityScoreConfig>,
) {
    loop {
        let _ = signal.wait().await;
        let config = quality_score_config.lock().await;
        if let Err(e) = storage.set_score_config(&*config).await {
            error!("Failed to set score config: {:?}", e);
        }
    }
}
