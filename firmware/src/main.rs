#![no_std]
#![no_main]

use {defmt_rtt as _, panic_probe as _}; // global logger + panicking-behavior

use defmt::*;
use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};

mod hardware;
use hardware::Hardware;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    // Initialize the hardware peripherals
    let Hardware { i2c: _ } = Hardware::default();

    info!("Hello World!");

    loop {
        Timer::after(Duration::from_secs(1)).await;
        None::<u32>.unwrap(); // This will cause a panic, which will be caught by the panic handler
    }
}
