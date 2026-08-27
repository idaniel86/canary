#![no_std]
#![no_main]

use core::str::FromStr;

use {defmt_rtt as _, panic_probe as _}; // global logger + panicking-behavior

use bme688;
use defmt::*;
use embassy_executor::Spawner;
use embassy_sync::{
    blocking_mutex::raw::NoopRawMutex,
    channel::{Channel, DynamicReceiver, DynamicSender},
};
use embassy_time::Timer;
use embedded_io_async::Write;
use micropb::MessageEncode;
use opt3001;

mod hardware;
use embassy_embedded_hal::shared_bus::asynch::i2c::I2cDevice;
use hardware::{Ethernet, Hardware, I2cBus};

mod proto {
    #![allow(clippy::all)]
    #![allow(nonstandard_style, unused, irrefutable_let_patterns)]
    include!(concat!(env!("OUT_DIR"), "/readings.rs"));
}

use proto::readings_ as readings;

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

    static SENSOR_CHANNEL: static_cell::StaticCell<
        Channel<NoopRawMutex, readings::SensorReading, 1>,
    > = static_cell::StaticCell::new();
    let sensor_channel = SENSOR_CHANNEL.init(Channel::new());
    let sensor_sender = sensor_channel.dyn_sender();
    let sensor_receiver = sensor_channel.dyn_receiver();

    // Spawn tasks
    spawner.spawn(opt3001_task(&i2c_bus, sensor_sender.clone()).unwrap());
    spawner.spawn(bme688_task(&i2c_bus, sensor_sender.clone()).unwrap());
    spawner.spawn(scd41_task(&i2c_bus, sensor_sender.clone()).unwrap());
    spawner.spawn(net_task(net_runner).unwrap());

    // Ensure DHCP configuration is up before trying connect
    net_stack.wait_config_up().await;
    info!(
        "Network stack is up. IP address: {}",
        net_stack.config_v4().unwrap().address
    );

    static RX_BUFFER: static_cell::StaticCell<[u8; 1024]> = static_cell::StaticCell::new();
    static TX_BUFFER: static_cell::StaticCell<[u8; 1024]> = static_cell::StaticCell::new();
    let rx_buffer = RX_BUFFER.init([0; 1024]);
    let tx_buffer = TX_BUFFER.init([0; 1024]);
    let mut socket = embassy_net::tcp::TcpSocket::new(net_stack, rx_buffer, tx_buffer);
    socket.set_timeout(Some(embassy_time::Duration::from_secs(10)));

    const SERVER_ADDRESS: Option<&str> = option_env!("SERVER_ADDRESS");

    let server_address = SERVER_ADDRESS
        .and_then(|server_address| core::net::SocketAddrV4::from_str(server_address).ok());
    if let Some(server_address) = server_address {
        spawner.spawn(tcp_client_task(socket, server_address, sensor_receiver).unwrap());
    } else {
        error!("SERVER_ADDRESS environment variable is not set or invalid");
    }

    loop {
        info!("Heartbeat...");
        Timer::after(embassy_time::Duration::from_secs(60)).await;
    }
}

#[embassy_executor::task]
async fn opt3001_task(
    i2c_bus: &'static I2cBus<'static>,
    sender: DynamicSender<'static, readings::SensorReading>,
) {
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
            let reading = readings::SensorReading::default().init_lux(lux as u32);
            sender.send(reading).await;
        }
    }
}

#[embassy_executor::task]
async fn bme688_task(
    i2c_bus: &'static I2cBus<'static>,
    sender: DynamicSender<'static, readings::SensorReading>,
) {
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
        + embassy_time::Duration::from_millis(280)) * 3;

    loop {
        Timer::after(duration).await;
        
        if let Ok(measurements) = sensor
            .get_measurements()
            .await
            .map_err(|e| error!("Error reading BME688 measurements: {:?}", e))
        {
            for measurement in measurements.iter() {
                let mut reading = readings::SensorReading::default()
                    .init_pressure(measurement.pressure as u32);
                let _ = reading.gas_resistance.push(readings::GasResistance {
                    profile: measurement.gas_meas_index as u32,
                    resistance: measurement.gas_resistance as u32,
                });
                sender.send(reading).await;
            }
        }
    }
}

#[embassy_executor::task]
async fn scd41_task(
    i2c_bus: &'static I2cBus<'static>,
    sender: DynamicSender<'static, readings::SensorReading>,
) {
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
                    let reading = readings::SensorReading::default()
                        .init_co2(measurement.co2 as u32)
                        .init_temperature(measurement.temperature)
                        .init_humidity(measurement.humidity);
                    sender.send(reading).await;
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
async fn tcp_client_task(
    mut socket: embassy_net::tcp::TcpSocket<'static>,
    remote_endpoint: core::net::SocketAddrV4,
    receiver: DynamicReceiver<'static, readings::SensorReading>,
) -> ! {
    let ip_address = embassy_net::Ipv4Address::from(*remote_endpoint.ip());
    let port = remote_endpoint.port();
    const CAPACITY: usize = 4 + micropb::size::max_encoded_size::<readings::SensorReading>();

    loop {
        match socket.connect((ip_address, port)).await {
            Ok(_) => {
                info!("Connected to server at {}", remote_endpoint);

                loop {
                    let reading = receiver.receive().await;
                    let mut encoder = micropb::PbEncoder::new(heapless::Vec::<u8, CAPACITY>::new());
                    match reading.encode_len_delimited(&mut encoder) {
                        Ok(_) => {
                            if let Err(e) = socket.write_all(encoder.as_writer()).await {
                                error!("Failed to send data: {:?}", e);
                                break;
                            }
                        }
                        Err(e) => {
                            error!("Failed to encode message: {:?}", e);
                        }
                    }
                }
            }
            Err(e) => {
                error!(
                    "Failed to connect to server at {}: {:?}",
                    remote_endpoint, e
                );
                Timer::after(embassy_time::Duration::from_secs(5)).await;
                continue;
            }
        }
    }
}
