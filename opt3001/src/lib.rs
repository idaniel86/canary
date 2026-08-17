#![cfg_attr(not(test), no_std)]

#[cfg(not(feature = "registers"))]
mod registers;
#[cfg(feature = "registers")]
pub mod registers;

mod math;
use math::{from_lux, to_lux};
pub use registers::{
    ComparisonMode, ConversionMode, ConversionTime, FaultCount, InterruptPolarity, LuxRange,
};

pub const MANUFACTURER_ID: u16 = 0x5449;
pub const DEVICE_ID: u16 = 0x3001;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SlaveAddress {
    /// Primary slave address (0x44)
    Primary,
    /// Secondary slave address (0x45, 0x46, or 0x47)
    Secondary(u8),
}

impl From<SlaveAddress> for u8 {
    fn from(address: SlaveAddress) -> Self {
        match address {
            SlaveAddress::Primary => 0x44,
            SlaveAddress::Secondary(value) => value & 0b11 | 0x44,
        }
    }
}

impl Default for SlaveAddress {
    fn default() -> Self {
        SlaveAddress::Primary
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Status {
    /// The current conversion mode of the OPT3001 sensor.
    pub conversion_mode: ConversionMode,
    /// Indicates whether an overflow condition has occurred in the data conversion process.
    pub is_overflow: bool,
    /// Indicates whether a conversion result is ready to be read from the result register.
    pub is_conversion_ready: bool,
    /// Indicates whether the result is above the high limit set in the high limit register.
    pub is_result_high: bool,
    /// Indicates whether the result is below the low limit set in the low limit register.
    pub is_result_low: bool,
}

impl From<registers::Config> for Status {
    fn from(config: registers::Config) -> Self {
        Status {
            conversion_mode: config.conversion_mode(),
            is_overflow: config.overflow(),
            is_conversion_ready: config.conversion_ready(),
            is_result_high: config.high(),
            is_result_low: config.low(),
        }
    }
}

/// Configuration for the interrupt reporting mechanisms of the OPT3001 sensor.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct InterruptConfig {
    polarity: InterruptPolarity,
    comparison_mode: ComparisonMode,
    fault_count: FaultCount,
    low_limit: registers::LowLimit,
    high_limit: registers::HighLimit,
}

impl InterruptConfig {
    pub fn builder() -> InterruptConfigBuilder {
        InterruptConfigBuilder::new()
    }
}

/// Builder pattern for constructing an `InterruptConfig` instance with a fluent interface.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct InterruptConfigBuilder {
    config: InterruptConfig,
    end_of_conversion_interrupt: bool,
}

impl InterruptConfigBuilder {
    /// Creates a new instance of the `InterruptConfigBuilder`.
    ///
    /// This method initializes the builder with default values for the interrupt configuration.
    /// The default values are:
    /// - Polarity: Active low
    /// - Comparison mode: Latched window
    /// - Fault count: 1 consecutive fault
    /// - Low limit: 0 lux
    /// - High limit: 0 lux
    /// - End-of-conversion interrupt: Disabled
    /// # Returns
    /// A new instance of the `InterruptConfigBuilder`.
    pub fn new() -> Self {
        Default::default()
    }

    /// Sets the polarity of the interrupt pin (INT) for the OPT3001 sensor.
    ///
    /// The polarity determines whether the interrupt pin is active high or active low when an interrupt condition occurs.
    /// # Arguments
    /// * `polarity` - The desired polarity for the interrupt pin.
    /// # Returns
    /// The updated `InterruptConfigBuilder` instance with the new polarity configuration.
    pub fn polarity(mut self, polarity: InterruptPolarity) -> Self {
        self.config.polarity = polarity;
        self
    }

    /// Sets the comparison mode for the interrupt reporting mechanisms.
    ///
    /// The comparison mode determines how the device evaluates the sensor readings against the configured low and high limit thresholds to trigger an interrupt. The available modes are:
    /// - Transparent Hysteresis: In this mode, the device continuously compares the sensor readings against the low and high limit thresholds. An interrupt is triggered when the reading crosses either threshold, and the device will continue to generate interrupts as long as the readings remain outside the thresholds.
    /// - Latched Window: In this mode, the device generates an interrupt when the sensor reading crosses the low or high limit thresholds, but the interrupt remains latched until the configuration register is read or the configuration register is written with a non-shutdown parameter
    ///
    /// # Arguments
    /// * `mode` - The desired comparison mode to set.
    /// # Returns
    /// The updated `InterruptConfigBuilder` instance with the new comparison mode configuration.
    pub fn comparison_mode(mut self, mode: ComparisonMode) -> Self {
        self.config.comparison_mode = mode;
        self
    }

    /// Sets the number of consecutive fault events necessary to trigger an interrupt.
    ///
    /// The fault count field instructs the device as to how many consecutive fault events are required
    /// to trigger the interrupt reporting mechanisms: the INT pin, the flag high field (FH), and flag low
    /// field (FL).
    ///
    /// # Arguments
    /// * `count` - The desired fault count to set.
    /// # Returns
    /// The updated `InterruptConfigBuilder` instance with the new fault count configuration.
    pub fn fault_count(mut self, count: FaultCount) -> Self {
        self.config.fault_count = count;
        self
    }

    /// Sets the low limit threshold for the OPT3001 sensor.
    ///
    /// The low limit register is used in conjunction with the high limit register to define a window
    /// for the latched window comparison mode or to set a threshold for the transparent hysteresis comparison mode.
    ///
    /// # Arguments
    /// * `exponent` - The exponent part of the low limit value.
    /// * `fractional_result` - The mantissa part of the low limit value.
    /// # Returns
    /// The updated `InterruptConfigBuilder` instance with the new low limit configuration.
    pub fn low_limit(mut self, exponent: u8, fractional_result: u16) -> Self {
        self.config.low_limit.set_exponent(exponent);
        self.config.low_limit.set_result(fractional_result);
        self
    }

    /// Sets the low limit threshold for the OPT3001 sensor based on a lux value.
    ///
    /// The low limit register is used in conjunction with the high limit register to define a window
    /// for the latched window comparison mode or to set a threshold for the transparent hysteresis comparison mode.
    ///
    /// # Arguments
    /// * `lux` - The lux value to set as the low limit threshold.
    /// # Returns
    /// The updated `InterruptConfigBuilder` instance with the new low limit configuration.
    pub fn low_limit_from_lux(self, lux: f32) -> Self {
        let (exponent, fractional_result) = from_lux(lux);
        self.low_limit(exponent, fractional_result)
    }

    /// Sets the high limit threshold for the OPT3001 sensor.
    ///
    /// The high limit register is used in conjunction with the low limit register to define a window
    /// for the latched window comparison mode or to set a threshold for the transparent hysteresis comparison mode.
    ///
    /// # Arguments
    /// * `exponent` - The exponent part of the high limit value.
    /// * `fractional_result` - The mantissa part of the high limit value.
    /// # Returns
    /// The updated `InterruptConfigBuilder` instance with the new high limit configuration.
    pub fn high_limit(mut self, exponent: u8, fractional_result: u16) -> Self {
        self.config.high_limit.set_exponent(exponent);
        self.config.high_limit.set_result(fractional_result);
        self
    }

    /// Sets the high limit threshold for the OPT3001 sensor based on a lux value.
    ///
    /// The high limit register is used in conjunction with the low limit register to define a window
    /// for the latched window comparison mode or to set a threshold for the transparent hysteresis comparison mode.
    /// # Arguments
    /// * `lux` - The lux value to set as the high limit threshold.
    /// # Returns
    /// The updated `InterruptConfigBuilder` instance with the new high limit configuration.
    pub fn high_limit_from_lux(self, lux: f32) -> Self {
        let (exponent, fractional_result) = from_lux(lux);
        self.high_limit(exponent, fractional_result)
    }

    /// Enables or disables the end-of-conversion interrupt.
    ///
    /// An end-of-conversion indicator mode can be used when every measurement is desired to be read by the
    /// processor, prompted by the INT pin going active on every measurement completion. This mode is entered by
    /// setting the most significant two bits of the low-limit register (LE[3:2] from the Low-Limit Register) to 11b. This
    /// end-of-conversion mode is typically used in conjunction with the latched window-style comparison mode. The INT
    /// pin becomes inactive when the configuration register is read or the configuration register is written with a non-
    /// shutdown parameter
    pub fn end_of_conversion_interrupt(mut self, enable: bool) -> Self {
        self.end_of_conversion_interrupt = enable;
        self
    }

    /// Finalizes the construction of the `InterruptConfig` instance.
    ///
    /// # Returns
    /// The constructed `InterruptConfig` instance.
    pub fn build(mut self) -> InterruptConfig {
        if self.end_of_conversion_interrupt {
            self.config
                .low_limit
                .set_exponent(self.config.low_limit.exponent() | 0b1100);
        }
        self.config
    }
}

pub struct Opt3001<I2C>
where
    I2C: embedded_hal_async::i2c::I2c,
{
    i2c: I2C,
    address: u8,
}

impl<I2C> Opt3001<I2C>
where
    I2C: embedded_hal_async::i2c::I2c,
{
    /// Creates a new instance of the `Opt3001` sensor driver.
    ///
    /// # Arguments
    /// * `i2c` - The I2C interface to communicate with the sensor.
    /// * `address` - The I2C slave address of the sensor.
    /// # Returns
    /// A new instance of the `Opt3001` sensor driver.
    pub fn new(i2c: I2C, address: SlaveAddress) -> Self {
        Opt3001 {
            i2c,
            address: address.into(),
        }
    }

    /// Reads the manufacturer ID from the OPT3001 sensor.
    ///
    /// # Returns
    /// A result containing the manufacturer ID if successful, or an error otherwise.
    pub async fn get_manufacturer_id(&mut self) -> Result<u16, I2C::Error> {
        registers::read_manufacturer_id(&mut self.i2c, self.address).await
    }

    /// Reads the device ID from the OPT3001 sensor.
    ///
    /// # Returns
    /// A result containing the device ID if successful, or an error otherwise.
    pub async fn get_device_id(&mut self) -> Result<u16, I2C::Error> {
        registers::read_device_id(&mut self.i2c, self.address).await
    }

    /// Reads the status of the OPT3001 sensor.
    ///
    /// # Returns
    /// A result containing the status of the sensor, or an error if the read operation failed.
    pub async fn get_status(&mut self) -> Result<Status, I2C::Error> {
        let status = registers::read_config(&mut self.i2c, self.address)
            .await?
            .into();
        Ok(status)
    }

    /// Reads the result register from the OPT3001 sensor.
    ///
    /// # Returns
    /// A result containing the calculated lux value if the conversion is ready, or an error if the read operation fails.
    pub async fn get_result(&mut self) -> Result<f32, I2C::Error> {
        self.get_result_raw()
            .await
            .map(|(exponent, mantissa)| to_lux(exponent, mantissa))
    }

    /// Reads the raw result register from the OPT3001 sensor.
    ///
    /// # Returns
    /// A result containing a tuple of the exponent and fractional result if the conversion is ready, or an error if the read operation fails.
    pub async fn get_result_raw(&mut self) -> Result<(u8, u16), I2C::Error> {
        let result = registers::read_result(&mut self.i2c, self.address).await?;
        Ok((result.exponent(), result.result()))
    }

    /// Sets the lux range of the OPT3001 sensor.
    ///
    /// The automatic full-scale setting mode allows the sensor to adjust its range based on the ambient light conditions,
    /// while the manual mode allows the user to specify a fixed range.
    ///
    /// ***Automatic full-scale setting mode:***
    ///
    /// The first measurement that the device takes in auto-range mode is a 10-ms range assessment measurement.
    /// The device then determines the appropriate full-scale range to take its first full measurement.
    /// For subsequent measurements, the full-scale range is set by the result of the previous measurement. If a
    /// measurement is towards the low side of full-scale, the full-scale range is decreased by one or two settings for the
    /// next measurement. If a measurement is towards the upper side of full-scale, the full-scale range is increased by
    /// one setting for the next measurement.
    /// If the measurement exceeds the full-scale range, resulting from a fast increasing optical transient event, the
    /// current measurement is aborted. This invalid measurement is not reported. A 10-ms measurement is taken to
    /// assess and properly reset the full-scale range. Then, a new measurement is taken with this proper full-scale
    /// range. Therefore, during a fast increasing optical transient in this mode, a measurement can possibly take longer
    /// to complete and report than indicated by the configuration register conversion time field.
    ///
    /// # Arguments
    /// * `range` - The desired lux range to set.
    /// # Returns
    /// A result indicating success or failure of the operation.
    pub async fn set_lux_range(&mut self, range: LuxRange) -> Result<(), I2C::Error> {
        let mut config = registers::read_config(&mut self.i2c, self.address).await?;
        config.set_lux_range(range);
        registers::write_config(&mut self.i2c, self.address, config).await
    }

    /// Sets the mask exponent bit of the OPT3001 sensor.
    ///
    /// The mask exponent field forces the result register exponent field (register 00h, bits E[3:0]) to
    /// 0000b when the full-scale range is manually set, which can simplify the processing of the
    /// result register when the full-scale range is manually programmed. This behavior occurs when
    /// the mask exponent field is set to 1 and the range number field (RN[3:0]) is set to less than
    /// 1100b. Note that the masking is only performed to the result register. When using the interrupt
    /// reporting mechanisms, the result comparison with the low-limit and high-limit registers is
    /// unaffected by the ME field.
    ///
    /// # Arguments
    /// * `mask` - A boolean indicating whether to enable (true) or disable (false) the mask exponent feature.
    /// # Returns
    /// A result indicating success or failure of the operation.
    pub async fn set_mask_exponent(&mut self, mask: bool) -> Result<(), I2C::Error> {
        let mut config = registers::read_config(&mut self.i2c, self.address).await?;
        config.set_mask_exponent(mask);
        registers::write_config(&mut self.i2c, self.address, config).await
    }

    /// Sets the conversion time of the OPT3001 sensor.
    ///
    /// The conversion time field determines the length of the light to digital conversion process. The
    /// choices are 100 ms and 800 ms. A longer integration time allows for a lower noise measurement.
    /// The conversion time also relates to the effective resolution of the data conversion process. The
    /// 800-ms conversion time allows for the fully specified lux resolution. The 100-ms conversion
    /// time with full-scale ranges above 0101b for E[3:0] in the result and configuration registers also
    /// allows for the fully specified lux resolution. The 100-ms conversion time with full-scale ranges
    /// below and including 0101b for E[3:0] can reduce the effective result resolution by up to three
    /// bits, as a function of the selected full-scale range. Range 0101b reduces by one bit. Ranges
    /// 0100b, 0011b, 0010b, and 0001b reduces by two bits. Range 0000b reduces by three bits.
    /// The result register format and associated LSB weight does not change as a function of the
    /// conversion time.
    ///
    /// # Arguments
    /// * `time` - The desired conversion time to set.
    /// # Returns
    /// A result indicating success or failure of the operation.
    pub async fn set_conversion_time(&mut self, time: ConversionTime) -> Result<(), I2C::Error> {
        let mut config = registers::read_config(&mut self.i2c, self.address).await?;
        config.set_conversion_time(time);
        registers::write_config(&mut self.i2c, self.address, config).await
    }

    /// Sets the interrupt configuration of the OPT3001 sensor.
    ///
    /// The interrupt configuration allows the user to customize the behavior of the interrupt pin (INT) based on the sensor's readings.
    /// The configuration includes setting the polarity of the interrupt pin, the comparison mode for interrupt reporting, the number of consecutive fault events required to trigger an interrupt, and the low and high limit thresholds for the sensor readings.
    ///
    /// # Arguments
    /// * `config` - The desired interrupt configuration to set.
    /// # Returns
    /// A result indicating success or failure of the operation.
    pub async fn set_interrupt_config(
        &mut self,
        interrupt_config: InterruptConfig,
    ) -> Result<(), I2C::Error> {
        registers::write_low_limit(&mut self.i2c, self.address, interrupt_config.low_limit).await?;
        registers::write_high_limit(&mut self.i2c, self.address, interrupt_config.high_limit)
            .await?;

        let mut config = registers::read_config(&mut self.i2c, self.address).await?;
        config.set_polarity(interrupt_config.polarity);
        config.set_comparison_mode(interrupt_config.comparison_mode.into());
        config.set_fault_count(interrupt_config.fault_count.into());
        registers::write_config(&mut self.i2c, self.address, config).await
    }

    /// Sets the conversion mode of the OPT3001 sensor.
    ///
    /// The mode of conversion operation field controls whether the device is operating in continuous
    /// conversion, single-shot, or low-power shutdown mode. The default is 00b (shutdown mode),
    /// such that upon power-up, the device only consumes operational level power after appropriately
    /// programming the device.
    /// When single-shot mode is selected by writing 01b to this field, the field continues to read 01b
    /// while the device is actively converting. When the single-shot conversion is complete, the mode
    /// of conversion operation field is automatically set to 00b and the device is shut down.
    /// When the device enters shutdown mode, either by completing a single-shot conversion or by a
    /// manual write to the configuration register, there is no change to the state of the reporting flags
    /// (conversion ready, flag high, flag low) or the INT pin. These signals are retained for
    /// subsequent read operations while the device is in shutdown mode.
    ///
    /// # Arguments
    /// * `mode` - The desired conversion mode to set.
    /// # Returns
    /// A result indicating success or failure of the operation.
    pub async fn set_conversion_mode(&mut self, mode: ConversionMode) -> Result<(), I2C::Error> {
        let mut config = registers::read_config(&mut self.i2c, self.address).await?;
        config.set_conversion_mode(mode.into());
        registers::write_config(&mut self.i2c, self.address, config).await
    }

    /// Releases the I2C interface back to the caller.
    ///
    /// # Returns
    /// The I2C interface that was used by the OPT3001 driver.
    pub fn release(self) -> I2C {
        self.i2c
    }
}
