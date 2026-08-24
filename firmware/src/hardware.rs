use embassy_stm32::{Config, bind_interrupts, dma, i2c, mode, peripherals, rcc, time::Hertz};
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
        let mut config = Config::default();
        // Disable HSI, since we have an external clock source (ST-Link MCO)
        config.rcc.hsi = None;
        // Use HSE bypass mode, since we have an external clock source (ST-Link MCO)
        config.rcc.hse = Some(rcc::Hse {
            freq: Hertz(8_000_000),
            mode: rcc::HseMode::BypassDigital,
        });
        // Configure PLL1 to generate 250 MHz system clock from HSE
        config.rcc.pll1 = Some(rcc::Pll {
            source: rcc::PllSource::HSE,   // Use HSE as PLL source
            prediv: rcc::PllPreDiv::DIV2,  // Pre-divide HSE by 2 (8 MHz / 2 = 4 MHz)
            mul: rcc::PllMul::MUL125,      // Multiply by 125 (4 MHz * 125 = 500 MHz)
            divp: Some(rcc::PllDiv::DIV2), // Divide by 2 to get 250 MHz
            divq: None,
            divr: None,
        });
        // Set AHB prescaler to 1 (250 MHz)
        config.rcc.ahb_pre = rcc::AHBPrescaler::DIV1;
        // Set APB1, APB2, and APB3 prescalers to 1 (250 MHz)
        config.rcc.apb1_pre = rcc::APBPrescaler::DIV1;
        config.rcc.apb2_pre = rcc::APBPrescaler::DIV1;
        config.rcc.apb3_pre = rcc::APBPrescaler::DIV1;
        // Use PLL1_P as system clock
        config.rcc.sys = rcc::Sysclk::PLL1_P;
        // Set voltage scale to Scale0 for maximum frequency
        config.rcc.voltage_scale = rcc::VoltageScale::Scale0;

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
