pub const REG_RESULT: u8 = 0x00;
pub const REG_CONFIG: u8 = 0x01;
pub const REG_LOW_LIMIT: u8 = 0x02;
pub const REG_HIGH_LIMIT: u8 = 0x03;
pub const REG_MANUFACTURER_ID: u8 = 0x7E;
pub const REG_DEVICE_ID: u8 = 0x7F;

/// Lux range
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LuxRange {
    /// Custom full-scale setting mode
    Manual(u8),
    /// Automatic full-scale setting mode
    Automatic,
}

impl Default for LuxRange {
    fn default() -> Self {
        LuxRange::Automatic
    }
}

impl From<LuxRange> for u8 {
    fn from(range: LuxRange) -> Self {
        match range {
            LuxRange::Manual(value) => value & 0b1111,
            LuxRange::Automatic => 0b1100,
        }
    }
}

impl From<u8> for LuxRange {
    fn from(value: u8) -> Self {
        match value & 0b1111 {
            0b1100 => LuxRange::Automatic,
            manual => LuxRange::Manual(manual),
        }
    }
}

/// Conversion time
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConversionTime {
    /// 100 ms conversion time
    Ms100,
    /// 800 ms conversion time
    Ms800,
}

impl Default for ConversionTime {
    fn default() -> Self {
        ConversionTime::Ms800
    }
}

impl From<ConversionTime> for u8 {
    fn from(ct: ConversionTime) -> Self {
        match ct {
            ConversionTime::Ms100 => 0b0,
            ConversionTime::Ms800 => 0b1,
        }
    }
}

impl From<u8> for ConversionTime {
    fn from(value: u8) -> Self {
        match value & 0b1 {
            0b0 => ConversionTime::Ms100,
            0b1 => ConversionTime::Ms800,
            _ => unreachable!(),
        }
    }
}

/// Mode of conversion operation
///
/// Determines whether the device is operating in continuous conversion, single-shot, or shutdown (low-power) mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConversionMode {
    /// Shutdown mode
    Shutdown,
    /// Single-shot conversion mode
    SingleShot,
    /// Continuous conversion mode
    Continuous,
}

impl From<ConversionMode> for u8 {
    fn from(mode: ConversionMode) -> Self {
        match mode {
            ConversionMode::Shutdown => 0b00,
            ConversionMode::SingleShot => 0b01,
            ConversionMode::Continuous => 0b10,
        }
    }
}

impl From<u8> for ConversionMode {
    fn from(value: u8) -> Self {
        match value & 0b11 {
            0b00 => ConversionMode::Shutdown,
            0b01 => ConversionMode::SingleShot,
            _ => ConversionMode::Continuous,
        }
    }
}

/// Fault count
///
/// Number of consecutive fault events necessary to trigger interrupt.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FaultCount {
    /// 1 consecutive fault
    One,
    /// 2 consecutive faults
    Two,
    /// 4 consecutive faults
    Four,
    /// 8 consecutive faults
    Eight,
}

impl Default for FaultCount {
    fn default() -> Self {
        FaultCount::One
    }
}

impl From<FaultCount> for u8 {
    fn from(fault_count: FaultCount) -> Self {
        match fault_count {
            FaultCount::One => 0b00,
            FaultCount::Two => 0b01,
            FaultCount::Four => 0b10,
            FaultCount::Eight => 0b11,
        }
    }
}

impl From<u8> for FaultCount {
    fn from(value: u8) -> Self {
        match value & 0b11 {
            0b00 => FaultCount::One,
            0b01 => FaultCount::Two,
            0b10 => FaultCount::Four,
            0b11 => FaultCount::Eight,
            _ => unreachable!(),
        }
    }
}

/// Interrupt pin polarity (active state)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InterruptPolarity {
    /// Active low interrupt polarity
    Low,
    /// Active high interrupt polarity
    High,
}

impl Default for InterruptPolarity {
    fn default() -> Self {
        InterruptPolarity::Low
    }
}

impl From<InterruptPolarity> for u8 {
    fn from(polarity: InterruptPolarity) -> Self {
        match polarity {
            InterruptPolarity::Low => 0b0,
            InterruptPolarity::High => 0b1,
        }
    }
}

impl From<u8> for InterruptPolarity {
    fn from(value: u8) -> Self {
        match value & 0b1 {
            0b0 => InterruptPolarity::Low,
            0b1 => InterruptPolarity::High,
            _ => unreachable!(),
        }
    }
}

/// Comparison mode for interrupt reporting mechanisms
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComparisonMode {
    /// Transparent hysteresis comparison mode
    TransparentHysteresis,
    /// Latched window comparison mode
    LatchedWindow,
}

impl Default for ComparisonMode {
    fn default() -> Self {
        ComparisonMode::TransparentHysteresis
    }
}

impl From<ComparisonMode> for u8 {
    fn from(mode: ComparisonMode) -> Self {
        match mode {
            ComparisonMode::TransparentHysteresis => 0b0,
            ComparisonMode::LatchedWindow => 0b1,
        }
    }
}

impl From<u8> for ComparisonMode {
    fn from(value: u8) -> Self {
        match value & 0b1 {
            0b0 => ComparisonMode::TransparentHysteresis,
            0b1 => ComparisonMode::LatchedWindow,
            _ => unreachable!(),
        }
    }
}

bitfield::bitfield! {
    /// OPT3001 result register
    #[derive(Clone, Copy, PartialEq, Eq)]
    pub struct Result(u16);
    impl Debug;
    /// These bits are the exponent bits.
    pub u8, exponent, _: 15, 12;
    /// These bits are the result in straight binary coding (zero to full-scale).
    pub u16, result, _: 11, 0;
}

/// Default value at reset is 0x0000, which means exponent = 0 and result = 0.
impl Default for Result {
    fn default() -> Self {
        Result(0)
    }
}

bitfield::bitfield! {
    /// OPT3001 configuration register
    #[derive(Clone, Copy, PartialEq, Eq)]
    pub struct Config(u16);
    impl Debug;
    /// The range number field selects the full-scale lux range of the device.
    pub u8, from into LuxRange, lux_range, set_lux_range: 15, 12;
    /// The conversion time field determines the length of the light to digital conversion process.
    /// A longer integration time allows for a lower noise measurement.
    pub u8, from into ConversionTime, conversion_time, set_conversion_time: 11, 11;
    /// The mode of conversion operation field controls whether the device is operating in continuous
    /// conversion, single-shot, or low-power shutdown mode.
    pub u8, from into ConversionMode, conversion_mode, set_conversion_mode: 10, 9;
    /// The overflow flag field indicates when an overflow condition occurs in the data conversion
    /// process, typically because the light illuminating the device exceeds the programmed full-scale
    /// range of the device.
    pub bool, overflow, _: 8;
    /// The conversion ready flag field indicates when a new conversion result is available in the result register.
    pub bool, conversion_ready, _: 7;
    /// The flag high field identifies that the result of a conversion is larger than a specified level of interest.
    pub bool, high, _: 6;
    /// The flag low field identifies that the result of a conversion is smaller than a specified level of interest.
    pub bool, low, _: 5;
    /// The latch field controls the functionality of the interrupt reporting mechanisms: the INT pin, the
    /// flag high field, and flag low field.
    pub u8, from into ComparisonMode, comparison_mode, set_comparison_mode: 4, 4;
    /// The polarity field controls the polarity or active state of the INT pin.
    /// 0 = The INT pin reports active low, pulling the pin low upon an interrupt event.
    /// 1 = Operation of the INT pin is inverted, where the INT pin reports active high, becoming high
    /// impedance and allowing the INT pin to be pulled high upon an interrupt event.
    pub u8, from into InterruptPolarity, polarity, set_polarity: 3, 3;
    /// The mask exponent field forces the result register exponent field to 0000b when the full-scale range is
    /// manually set, which can simplify the processing of the result register when the full-scale range is
    /// manually programmed.
    pub bool, mask_exponent, set_mask_exponent: 2;
    /// The fault count field instructs the device as to how many consecutive fault events are required
    /// to trigger the interrupt reporting mechanisms: the INT pin, the flag high field, and flag low field.
    pub u8, from into FaultCount, fault_count, set_fault_count: 1, 0;
}

/// Default value at reset is 0xC810, which means:
/// - lux_range = 1100b (automatic full-scale range)
/// - conversion_time = 1b (800 ms)
/// - mode_of_conversion_operation = 00b (shutdown)
/// - overflow = 0b (no overflow)
/// - conversion_ready = 0b (no conversion ready)
/// - high = 0b (no high)
/// - low = 0b (no low)
/// - comparison_mode = 1b (latched window)
/// - polarity = 0b (active low)
/// - mask_exponent = 0b (mask exponent disabled)
/// - fault_count = 0b (1 consecutive fault)
impl Default for Config {
    fn default() -> Self {
        Config(0xC810)
    }
}

bitfield::bitfield! {
    /// OPT3001 low limit register.
    /// Sets the lower comparison limit for the interrupt reporting mechanisms: the INT pin,
    /// the flag high field, and flag low field.
    #[derive(Clone, Copy, PartialEq, Eq)]
    pub struct LowLimit(u16);
    impl Debug;
    /// These bits are the exponent bits.
    pub u8, exponent, set_exponent: 15, 12;
    /// These bits are the result in straight binary coding (zero to full-scale).
    pub u16, result, set_result: 11, 0;
}

/// Default value at reset is 0x0000, which means exponent = 0 and result = 0.
impl Default for LowLimit {
    fn default() -> Self {
        LowLimit(0)
    }
}

bitfield::bitfield! {
    /// OPT3001 high limit register.
    /// Sets the upper comparison limit for the interrupt reporting mechanisms: the INT pin,
    /// the flag high field, and flag low field.
    #[derive(Clone, Copy, PartialEq, Eq)]
    pub struct HighLimit(u16);
    impl Debug;
    /// These bits are the exponent bits.
    pub u8, exponent, set_exponent: 15, 12;
    /// These bits are the result in straight binary coding (zero to full-scale).
    pub u16, result, set_result: 11, 0;
}

/// Default value at reset is 0xBFFF, which means exponent = 11 and result = 4095,
/// which is the maximum value for the result register.
impl Default for HighLimit {
    fn default() -> Self {
        HighLimit(0xBFFF)
    }
}

/// Reads a 16-bit value from the specified register of the OPT3001 device.
///
/// # Arguments
/// * `i2c` - The I2C interface to communicate with the device.
/// * `address` - The I2C address of the OPT3001 device.
/// * `register` - The register address to read from.
///
/// # Returns
/// A result containing the 16-bit value read from the register, or an error if the read operation fails.
async fn read_register<I2C>(
    i2c: &mut I2C,
    address: u8,
    register: u8,
) -> core::result::Result<u16, I2C::Error>
where
    I2C: embedded_hal_async::i2c::I2c,
{
    let mut buf = [0u8; 2];
    i2c.write_read(address, &[register], &mut buf).await?;
    Ok(u16::from_be_bytes(buf))
}

/// Writes a 16-bit value to the specified register of the OPT3001 device.
///
/// # Arguments
/// * `i2c` - The I2C interface to communicate with the device.
/// * `address` - The I2C address of the OPT3001 device.
/// * `register` - The register address to write to.
/// * `value` - The 16-bit value to write to the register.
///
/// # Returns
/// A result indicating success or failure of the write operation.
async fn write_register<I2C>(
    i2c: &mut I2C,
    address: u8,
    register: u8,
    value: u16,
) -> core::result::Result<(), I2C::Error>
where
    I2C: embedded_hal_async::i2c::I2c,
{
    let buf = [register, (value >> 8) as u8, value as u8];
    i2c.write(address, &buf).await
}

/// Reads the result register from the OPT3001 device.
///
/// # Arguments
/// * `i2c` - The I2C interface to communicate with the device.
/// * `address` - The I2C address of the OPT3001 device.
///
/// # Returns
/// A result containing the `registers::Result` struct if successful, or an error if the read operation fails.
pub async fn read_result<I2C>(
    i2c: &mut I2C,
    address: u8,
) -> core::result::Result<Result, I2C::Error>
where
    I2C: embedded_hal_async::i2c::I2c,
{
    let raw = read_register(i2c, address, REG_RESULT).await?;
    Ok(Result(raw))
}

/// Reads the configuration register from the OPT3001 device.
///
/// # Arguments
/// * `i2c` - The I2C interface to communicate with the device.
/// * `address` - The I2C address of the OPT3001 device.
///
/// # Returns
/// A result containing the `registers::Config` struct if successful, or an error if the read operation fails.
pub async fn read_config<I2C>(
    i2c: &mut I2C,
    address: u8,
) -> core::result::Result<Config, I2C::Error>
where
    I2C: embedded_hal_async::i2c::I2c,
{
    let raw = read_register(i2c, address, REG_CONFIG).await?;
    Ok(Config(raw))
}

/// Writes the configuration register to the OPT3001 device.
///
/// # Arguments
/// * `i2c` - The I2C interface to communicate with the device.
/// * `address` - The I2C address of the OPT3001 device.
/// * `config` - The `Config` struct containing the configuration to write.
///
/// # Returns
/// A result indicating success or failure of the write operation.
pub async fn write_config<I2C>(
    i2c: &mut I2C,
    address: u8,
    config: Config,
) -> core::result::Result<(), I2C::Error>
where
    I2C: embedded_hal_async::i2c::I2c,
{
    write_register(i2c, address, REG_CONFIG, config.0).await
}

/// Reads the manufacturer ID from the OPT3001 device.
///
/// # Arguments
/// * `i2c` - The I2C interface to communicate with the device.
/// * `address` - The I2C address of the OPT3001 device.
///
/// # Returns
/// A result containing the manufacturer ID if successful, or an error if the read operation fails.
pub async fn read_manufacturer_id<I2C>(
    i2c: &mut I2C,
    address: u8,
) -> core::result::Result<u16, I2C::Error>
where
    I2C: embedded_hal_async::i2c::I2c,
{
    read_register(i2c, address, REG_MANUFACTURER_ID).await
}

/// Reads the device ID from the OPT3001 device.
///
/// # Arguments
/// * `i2c` - The I2C interface to communicate with the device.
/// * `address` - The I2C address of the OPT3001 device.
///
/// # Returns
/// A result containing the device ID if successful, or an error if the read operation fails.
pub async fn read_device_id<I2C>(
    i2c: &mut I2C,
    address: u8,
) -> core::result::Result<u16, I2C::Error>
where
    I2C: embedded_hal_async::i2c::I2c,
{
    read_register(i2c, address, REG_DEVICE_ID).await
}

/// Reads the low limit register from the OPT3001 device.
///
/// # Arguments
/// * `i2c` - The I2C interface to communicate with the device.
/// * `address` - The I2C address of the OPT3001 device.
/// 
/// # Returns
/// A result containing the `registers::LowLimit` struct if successful, or an error if the read operation fails.
// Only used externally when the `registers` feature makes this module public.
#[cfg_attr(not(feature = "registers"), allow(dead_code))]
pub async fn read_low_limit<I2C>(
    i2c: &mut I2C,
    address: u8,
) -> core::result::Result<LowLimit, I2C::Error>
where
    I2C: embedded_hal_async::i2c::I2c,
{
    let raw = read_register(i2c, address, REG_LOW_LIMIT).await?;
    Ok(LowLimit(raw))
}

/// Writes the low limit register to the OPT3001 device.
///
/// # Arguments
/// * `i2c` - The I2C interface to communicate with the device.
/// * `address` - The I2C address of the OPT3001 device.
/// * `low_limit` - The `LowLimit` struct containing the low limit value to write.
///
/// # Returns
/// A result indicating success or failure of the write operation.
pub async fn write_low_limit<I2C>(
    i2c: &mut I2C,
    address: u8,
    low_limit: LowLimit,
) -> core::result::Result<(), I2C::Error>
where
    I2C: embedded_hal_async::i2c::I2c,
{
    write_register(i2c, address, REG_LOW_LIMIT, low_limit.0).await
}

/// Reads the high limit register from the OPT3001 device.
///
/// # Arguments
/// * `i2c` - The I2C interface to communicate with the device.
/// * `address` - The I2C address of the OPT3001 device.
///
/// # Returns
/// A result containing the `registers::HighLimit` struct if successful, or an error if the read operation fails.
// Only used externally when the `registers` feature makes this module public.
#[cfg_attr(not(feature = "registers"), allow(dead_code))]
pub async fn read_high_limit<I2C>(
    i2c: &mut I2C,
    address: u8,
) -> core::result::Result<HighLimit, I2C::Error>
where
    I2C: embedded_hal_async::i2c::I2c,
{
    let raw = read_register(i2c, address, REG_HIGH_LIMIT).await?;
    Ok(HighLimit(raw))
}

/// Writes the high limit register to the OPT3001 device.
///
/// # Arguments
/// * `i2c` - The I2C interface to communicate with the device.
/// * `address` - The I2C address of the OPT3001 device.
/// * `high_limit` - The `HighLimit` struct containing the high limit value to write.
///
/// # Returns
/// A result indicating success or failure of the write operation.
pub async fn write_high_limit<I2C>(
    i2c: &mut I2C,
    address: u8,
    high_limit: HighLimit,
) -> core::result::Result<(), I2C::Error>
where
    I2C: embedded_hal_async::i2c::I2c,
{
    write_register(i2c, address, REG_HIGH_LIMIT, high_limit.0).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use embedded_hal_mock::eh1::i2c::{Mock, Transaction};
    const ADDRESS: u8 = 0x44;

    #[test]
    fn test_low_limit_default() {
        let low_limit = LowLimit::default();
        assert_eq!(low_limit.0, 0x0000);
        assert_eq!(low_limit.exponent(), 0);
        assert_eq!(low_limit.result(), 0);
    }

    #[test]
    fn test_high_limit_default() {
        let high_limit = HighLimit::default();
        assert_eq!(high_limit.0, 0xBFFF);
        assert_eq!(high_limit.exponent(), 11);
        assert_eq!(high_limit.result(), 4095);
    }

    #[test]
    fn test_config_default() {
        let config = Config::default();
        assert_eq!(config.0, 0xC810);
        assert_eq!(config.lux_range(), LuxRange::Automatic);
        assert_eq!(config.conversion_time(), ConversionTime::Ms800);
        assert_eq!(config.conversion_mode(), ConversionMode::Shutdown);
        assert_eq!(config.overflow(), false);
        assert_eq!(config.conversion_ready(), false);
        assert_eq!(config.high(), false);
        assert_eq!(config.low(), false);
        assert_eq!(config.comparison_mode(), ComparisonMode::LatchedWindow);
        assert_eq!(config.polarity(), InterruptPolarity::Low);
        assert_eq!(config.mask_exponent(), false);
        assert_eq!(config.fault_count(), FaultCount::One);
    }

    #[tokio::test]
    async fn test_result() {
        let transaction = [Transaction::write_read(
            ADDRESS,
            vec![REG_RESULT],
            vec![0x34, 0x56],
        )];
        let mut i2c = Mock::new(&transaction);

        assert_eq!(
            read_result(&mut i2c, ADDRESS).await.unwrap(),
            Result(0x3456)
        );

        i2c.done();
    }

    #[tokio::test]
    async fn test_config_roundtrip() {
        let transaction = [
            Transaction::write(ADDRESS, vec![REG_CONFIG, 0x12, 0x34]),
            Transaction::write_read(ADDRESS, vec![REG_CONFIG], vec![0x12, 0x34]),
        ];
        let mut i2c = Mock::new(&transaction);

        write_config(&mut i2c, ADDRESS, Config(0x1234))
            .await
            .unwrap();
        assert_eq!(
            read_config(&mut i2c, ADDRESS).await.unwrap(),
            Config(0x1234)
        );

        i2c.done();
    }

    #[tokio::test]
    async fn test_low_limit_roundtrip() {
        let transaction = [
            Transaction::write(ADDRESS, vec![REG_LOW_LIMIT, 0x0F, 0xFF]),
            Transaction::write_read(ADDRESS, vec![REG_LOW_LIMIT], vec![0x0F, 0xFF]),
        ];
        let mut i2c = Mock::new(&transaction);

        write_low_limit(&mut i2c, ADDRESS, LowLimit(0x0FFF))
            .await
            .unwrap();
        assert_eq!(
            read_low_limit(&mut i2c, ADDRESS).await.unwrap(),
            LowLimit(0x0FFF)
        );

        i2c.done();
    }

    #[tokio::test]
    async fn test_high_limit_roundtrip() {
        let transaction = [
            Transaction::write(ADDRESS, vec![REG_HIGH_LIMIT, 0x0A, 0xBC]),
            Transaction::write_read(ADDRESS, vec![REG_HIGH_LIMIT], vec![0x0A, 0xBC]),
        ];
        let mut i2c = Mock::new(&transaction);

        write_high_limit(&mut i2c, ADDRESS, HighLimit(0x0ABC))
            .await
            .unwrap();
        assert_eq!(
            read_high_limit(&mut i2c, ADDRESS).await.unwrap(),
            HighLimit(0x0ABC)
        );

        i2c.done();
    }

    #[tokio::test]
    async fn test_manufacturer_id() {
        let transaction = [Transaction::write_read(
            ADDRESS,
            vec![REG_MANUFACTURER_ID],
            vec![0x54, 0x49],
        )];
        let mut i2c = Mock::new(&transaction);

        assert_eq!(
            read_manufacturer_id(&mut i2c, ADDRESS).await.unwrap(),
            0x5449
        );

        i2c.done();
    }

    #[tokio::test]
    async fn test_device_id() {
        let transaction = [Transaction::write_read(
            ADDRESS,
            vec![REG_DEVICE_ID],
            vec![0x30, 0x01],
        )];
        let mut i2c = Mock::new(&transaction);
        assert_eq!(read_device_id(&mut i2c, ADDRESS).await.unwrap(), 0x3001);

        i2c.done();
    }
}
