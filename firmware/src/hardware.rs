use embassy_stm32::{Config, bind_interrupts, dma, i2c, mode, peripherals};
use embassy_sync::{blocking_mutex::raw::NoopRawMutex, mutex::Mutex};

bind_interrupts!(struct Irqs {
    I2C2_EV => i2c::EventInterruptHandler<peripherals::I2C2>;
    I2C2_ER => i2c::ErrorInterruptHandler<peripherals::I2C2>;
    GPDMA1_CHANNEL4 => dma::InterruptHandler<peripherals::GPDMA1_CH4>;
    GPDMA1_CHANNEL5 => dma::InterruptHandler<peripherals::GPDMA1_CH5>;
});

/// A struct that holds the hardware peripherals used in the application
pub struct Hardware<'d> {
    pub i2c: SharedI2c<i2c::I2c<'d, mode::Async, i2c::Master>>,
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

        Hardware {
            i2c: SharedI2c::new(i2c),
        }
    }
}

/// A wrapper around an I2C interface that allows for shared access across multiple tasks
pub struct SharedI2c<I2C>
where
    I2C: embedded_hal_async::i2c::I2c,
{
    i2c: Mutex<NoopRawMutex, I2C>,
}

impl<I2C> SharedI2c<I2C>
where
    I2C: embedded_hal_async::i2c::I2c,
{
    /// Creates a new SharedI2c instance
    ///
    /// # Arguments
    /// * `i2c` - The I2C interface to be shared
    /// # Returns
    /// A new instance of SharedI2c
    pub fn new(i2c: I2C) -> Self {
        SharedI2c {
            i2c: Mutex::new(i2c),
        }
    }
}

impl<I2C> embedded_hal_async::i2c::ErrorType for SharedI2c<I2C>
where
    I2C: embedded_hal_async::i2c::I2c,
{
    type Error = I2C::Error;
}

impl<I2C> embedded_hal_async::i2c::I2c for SharedI2c<I2C>
where
    I2C: embedded_hal_async::i2c::I2c,
{
    async fn transaction(
        &mut self,
        address: embedded_hal_async::i2c::SevenBitAddress,
        operations: &mut [embedded_hal_async::i2c::Operation<'_>],
    ) -> Result<(), Self::Error> {
        let mut i2c = self.i2c.lock().await;
        i2c.transaction(address, operations).await
    }
}
