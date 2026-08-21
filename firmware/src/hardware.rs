use embassy_stm32::{Config, bind_interrupts, dma, i2c, mode, peripherals};
use embassy_sync::{blocking_mutex::raw::NoopRawMutex, mutex::Mutex};
use static_cell::StaticCell;

pub type I2cBus<'d> = Mutex<NoopRawMutex, i2c::I2c<'d, mode::Async, i2c::Master>>;
static I2C_BUS: StaticCell<I2cBus<'static>> = StaticCell::new();

bind_interrupts!(struct Irqs {
    I2C2_EV => i2c::EventInterruptHandler<peripherals::I2C2>;
    I2C2_ER => i2c::ErrorInterruptHandler<peripherals::I2C2>;
    GPDMA1_CHANNEL4 => dma::InterruptHandler<peripherals::GPDMA1_CH4>;
    GPDMA1_CHANNEL5 => dma::InterruptHandler<peripherals::GPDMA1_CH5>;
});

/// A struct that holds the hardware peripherals used in the application
pub struct Hardware<'d> {
    pub i2c_bus: &'d I2cBus<'static>,
}

impl<'d> Default for Hardware<'d> {
    fn default() -> Self {
        // Initialize the embassy runtime
        let config = Config::default();
        let p = embassy_stm32::init(config);

        // Initialize I2C2 in asynchronous master mode with DMA channels 4 and 5
        let i2c_config = i2c::Config::default();
        let i2c = i2c::I2c::new(
            p.I2C2,
            p.PF1,
            p.PF0,
            p.GPDMA1_CH4,
            p.GPDMA1_CH5,
            Irqs,
            i2c_config,
        );

        let i2c_bus = I2C_BUS.init(Mutex::<NoopRawMutex, _>::new(i2c));

        Hardware { i2c_bus }
    }
}
