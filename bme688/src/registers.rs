use crate::registers;

///! Register addresses for the BME688 sensor.

/// Register for 3rd group of coefficients
const REG_COEFF_3: u8 = 0x00;
/// 0th Field address
const REG_FIELD_0: u8 = 0x1D;
/// 0th Current DAC address
const REG_IDAC_HEAT_0: u8 = 0x50;
/// 0th Heater resistance address
const REG_RES_HEAT_0: u8 = 0x5A;
/// 0th Heater duration address
const REG_GAS_WAIT_0: u8 = 0x64;
/// Shared heating duration address
const REG_SHD_GAS_WAIT: u8 = 0x6E;
/// CTRL_GAS_0 address
const REG_CTRL_GAS_0: u8 = 0x70;
/// CTRL_GAS_1 address
const REG_CTRL_GAS_1: u8 = 0x71;
/// CTRL_HUM address
const REG_CTRL_HUM: u8 = 0x72;
/// CTRL_MEAS address
const REG_CTRL_MEAS: u8 = 0x74;
/// CONFIG address
const REG_CONFIG: u8 = 0x75;
/// Register for 1st group of coefficients
const REG_COEFF_1: u8 = 0x8A;
/// Chip ID address
const REG_CHIP_ID: u8 = 0xD0;
/// Software reset address
const REG_SOFT_RESET: u8 = 0xE0;
/// Register for 2nd group of coefficients
const REG_COEFF_2: u8 = 0xE1;
/// Variant ID address
const REG_VARIANT_ID: u8 = 0xF0;

///! Constants for lengths of various data structures in the BME688 sensor.

/// Length of the first group of coefficients
const LEN_COEFF_1: usize = 23;
/// Length of the second group of coefficients
const LEN_COEFF_2: usize = 14;
/// Length of the third group of coefficients
const LEN_COEFF_3: usize = 5;
/// Total length of all coefficient groups
pub(crate) const LEN_COEFF: usize = LEN_COEFF_1 + LEN_COEFF_2 + LEN_COEFF_3;
/// Length of the field data
pub(crate) const LEN_FIELD: usize = 17;
/// Length of the control gas register
pub(crate) const LEN_CTRL_GAS: usize = 2;
/// Maximum number of field data chunks
pub const MAX_FIELD_COUNT: usize = 3;
/// Maximum number of heater profiles
pub const MAX_HEATER_PROFILES: usize = 10;

/// Operating mode of sensor
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    /// 00: Sleep operation mode
    Sleep,
    /// 01: Forced operation mode
    Forced,
    /// 10: Parallel operation mode
    Parallel,
    /// 11: Sequential operation mode
    Sequential,
}

impl From<Mode> for u8 {
    fn from(mode: Mode) -> Self {
        match mode {
            Mode::Sleep => 0b00,
            Mode::Forced => 0b01,
            Mode::Parallel => 0b10,
            Mode::Sequential => 0b11,
        }
    }
}

impl From<u8> for Mode {
    fn from(value: u8) -> Self {
        match value & 0b11 {
            0b00 => Mode::Sleep,
            0b01 => Mode::Forced,
            0b10 => Mode::Parallel,
            0b11 => Mode::Sequential,
            _ => unreachable!(),
        }
    }
}

/// IIR filter applies only to temperature and pressure
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Filter {
    /// 000: Filter off
    #[default]
    Off,
    /// 001: Filter coefficient of 2
    Size1,
    /// 010: Filter coefficient of 4
    Size3,
    /// 011: Filter coefficient of 8
    Size7,
    /// 100: Filter coefficient of 16
    Size15,
    /// 101: Filter coefficient of 32
    Size31,
    /// 110: Filter coefficient of 64
    Size63,
    /// 111: Filter coefficient of 128
    Size127,
}

impl From<Filter> for u8 {
    fn from(filter: Filter) -> Self {
        match filter {
            Filter::Off => 0b000,
            Filter::Size1 => 0b001,
            Filter::Size3 => 0b010,
            Filter::Size7 => 0b011,
            Filter::Size15 => 0b100,
            Filter::Size31 => 0b101,
            Filter::Size63 => 0b110,
            Filter::Size127 => 0b111,
        }
    }
}

impl From<u8> for Filter {
    fn from(value: u8) -> Self {
        match value & 0b111 {
            0b000 => Filter::Off,
            0b001 => Filter::Size1,
            0b010 => Filter::Size3,
            0b011 => Filter::Size7,
            0b100 => Filter::Size15,
            0b101 => Filter::Size31,
            0b110 => Filter::Size63,
            0b111 => Filter::Size127,
            _ => unreachable!(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Oversampling {
    /// 000: Skipped
    #[default]
    Skipped,
    /// 001: Perform 1 measurement
    X1,
    /// 010: Perform 2 measurements
    X2,
    /// 011: Perform 4 measurements
    X4,
    /// 100: Perform 8 measurements
    X8,
    /// 101: Perform 16 measurements
    X16,
}

impl Oversampling {
    /// Get the number of cycles corresponding to the oversampling setting.
    ///
    /// # Returns
    /// The number of cycles for the oversampling setting.
    pub fn cycles(&self) -> u32 {
        match self {
            Oversampling::Skipped => 0,
            Oversampling::X1 => 1,
            Oversampling::X2 => 2,
            Oversampling::X4 => 4,
            Oversampling::X8 => 8,
            Oversampling::X16 => 16,
        }
    }
}

impl From<Oversampling> for u8 {
    fn from(oversampling: Oversampling) -> Self {
        match oversampling {
            Oversampling::Skipped => 0b000,
            Oversampling::X1 => 0b001,
            Oversampling::X2 => 0b010,
            Oversampling::X4 => 0b011,
            Oversampling::X8 => 0b100,
            Oversampling::X16 => 0b101,
        }
    }
}

impl From<u8> for Oversampling {
    fn from(value: u8) -> Self {
        match value & 0b111 {
            0b000 => Oversampling::Skipped,
            0b001 => Oversampling::X1,
            0b010 => Oversampling::X2,
            0b011 => Oversampling::X4,
            0b100 => Oversampling::X8,
            0b101 => Oversampling::X16,
            _ => Oversampling::X16,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StandbyTime {
    /// 0.59 ms.
    Ms0_59,
    /// 62.5 ms.
    Ms62_5,
    /// 125 ms.
    Ms125,
    /// 250 ms.
    Ms250,
    /// 500 ms.
    Ms500,
    /// 1,000 ms.
    Ms1000,
    /// 10 ms.
    Ms10,
    /// 20 ms.
    Ms20,
}

impl From<StandbyTime> for u8 {
    fn from(standby_time: StandbyTime) -> Self {
        match standby_time {
            StandbyTime::Ms0_59 => 0b000,
            StandbyTime::Ms62_5 => 0b001,
            StandbyTime::Ms125 => 0b010,
            StandbyTime::Ms250 => 0b011,
            StandbyTime::Ms500 => 0b100,
            StandbyTime::Ms1000 => 0b101,
            StandbyTime::Ms10 => 0b110,
            StandbyTime::Ms20 => 0b111,
        }
    }
}

impl From<u8> for StandbyTime {
    fn from(value: u8) -> Self {
        match value & 0b111 {
            0b000 => StandbyTime::Ms0_59,
            0b001 => StandbyTime::Ms62_5,
            0b010 => StandbyTime::Ms125,
            0b011 => StandbyTime::Ms250,
            0b100 => StandbyTime::Ms500,
            0b101 => StandbyTime::Ms1000,
            0b110 => StandbyTime::Ms10,
            0b111 => StandbyTime::Ms20,
            _ => unreachable!(),
        }
    }
}

// Calculate the bit offset for a given byte and bit position
fn offset(byte: usize, bit: usize) -> usize {
    byte * 8 + bit
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct H1Coeff(pub u16);

impl From<u16> for H1Coeff {
    fn from(value: u16) -> Self {
        let [lsb, msb] = value.to_le_bytes();
        H1Coeff(u16::from_be_bytes([msb, lsb << 4]) >> 4)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct H2Coeff(pub u16);

impl From<u16> for H2Coeff {
    fn from(value: u16) -> Self {
        H2Coeff(u16::from_be(value) >> 4)
    }
}

bitfield::bitfield! {
    #[derive(Clone, Copy, PartialEq, Eq)]
    /// Represents the calibration coefficients read from the BME688 sensor.
    pub(crate) struct Coeffs([u8]);
    impl Debug;
    // Coefficient T2 LSB 0x8A (bits 7–0), MSB 0x8B (bits 7–0)
    pub i16, t2, _: offset(1, 7), offset(0, 0);
    // Coefficient T3 0x8C (bits 7–0)
    pub i8, t3, _: offset(2, 7), offset(2, 0);
    // Coefficient P1 LSB 0x8E (bits 7–0), MSB 0x8F (bits 7–0)
    pub u16, p1, _: offset(5, 7), offset(4, 0);
    // Coefficient P2 LSB 0x90 (bits 7–0), P2 MSB 0x91 (bits 7–0)
    pub i16, p2, _: offset(7, 7), offset(6, 0);
    // Coefficient P3 0x92 (bits 7–0)
    pub i8, p3, _: offset(8, 7), offset(8, 0);
    // Coefficient P4 LSB 0x94 (bits 7–0), P4 MSB 0x95 (bits 7–0)
    pub i16, p4, _: offset(11, 7), offset(10, 0);
    // Coefficient P5 LSB 0x96 (bits 7–0), MSB 0x97 (bits 7–0)
    pub i16, p5, _: offset(13, 7), offset(12, 0);
    // Coefficient P7 0x98 (bits 7–0)
    pub i8, p7, _: offset(14, 7), offset(14, 0);
    // Coefficient P6 0x99 (bits 7–0)
    pub i8, p6, _: offset(15, 7), offset(15, 0);
    // Coefficient P8 0x9C (bits 7–0), MSB 0x9D (bits 7–0)
    pub i16, p8, _: offset(19, 7), offset(18, 0);
    // Coefficient P9 LSB 0x9E (bits 7–0),  0x9F (bits 7–0)
    pub i16, p9, _: offset(21, 7), offset(20, 0);
    // Coefficient P10 0xA0 (bits 7–0)
    pub u8, p10, _: offset(22, 7), offset(22, 0);
    // Coefficient H2 LSB 0xE2 (bits 7–4), MSB 0xE1 (bits 7–0)
    pub u16, from into H2Coeff, h2, _: offset(24, 7), offset(23, 0);
    /// Coefficient H1 LSB 0xE2 (bits 3–0), MSB 0xE3 (bits 7–0)
    pub u16, from into H1Coeff, h1, _: offset(25, 7), offset(24, 0);
    /// Coefficient H3 0xE4 (bits 7–0)
    pub i8, h3, _: offset(26, 7), offset(26, 0);
    /// Coefficient H4 0xE5 (bits 7–0)
    pub i8, h4, _: offset(27, 7), offset(27, 0);
    /// Coefficient H5 0xE6 (bits 7–0)
    pub i8, h5, _: offset(28, 7), offset(28, 0);
    /// Coefficient H6 0xE7 (bits 7–0)
    pub u8, h6, _: offset(29, 7), offset(29, 0);
    /// Coefficient H7 0xE8 (bits 7–0)
    pub i8, h7, _: offset(30, 7), offset(30, 0);
    /// Coefficient T1 LSB 0xE9 (bits 7–0), MSB 0xEA (bits 7–0)
    pub u16, t1, _: offset(32, 7), offset(31, 0);
    /// Coefficient G2 LSB 0xEB (bits 7–0), MSB 0xEC (bits 7–0)
    pub i16, g2, _: offset(34, 7), offset(33, 0);
    /// Coefficient G1 0xED (bits 7–0)
    pub i8, g1, _: offset(35, 7), offset(35, 0);
    /// Coefficient G3 0xEE (bits 7–0)
    pub i8, g3, _: offset(36, 7), offset(36, 0);
    /// Coefficient resistance heat value 0x00 (bits 7–0)
    pub i8, res_heat_val, _: offset(37, 7), offset(37, 0);
    /// Coefficient resistance heat range 0x02 (bits 1–0)
    pub u8, res_heat_range, _: offset(39, 1), offset(39, 0);
    /// Coefficient range switching error 0x04 (bits 7–0)
    pub i8, range_sw_error, _: offset(41, 7), offset(41, 0);
}

bitfield::bitfield! {
    /// Represents the configuration register (0x75) of the BME688 sensor.
    #[derive(Clone, Copy, PartialEq, Eq)]
    pub(crate) struct Config(u8);
    impl Debug;
    /// Standby time (bits 7–5)
    pub u8, from into StandbyTime, standby_time, set_standby_time: 7, 5;
    /// Filter coefficient (bits 4–2)
    pub u8, from into Filter, filter, set_filter: 4, 2;
}

bitfield::bitfield! {
    /// Represents the control measurement register (0x74) of the BME688 sensor.
    #[derive(Clone, Copy, PartialEq, Eq)]
    pub(crate) struct CtrlMeas(u8);
    impl Debug;
    // Oversampling of temperature (bits 7–5)
    pub u8, from into Oversampling, temp_os, set_temp_os: 7, 5;
    // Oversampling of pressure (bits 4–2)
    pub u8, from into Oversampling, press_os, set_press_os: 4, 2;
    // Operating mode (bits 1–0)
    pub u8, from into Mode, mode, set_mode: 1, 0;
}

bitfield::bitfield! {
    /// Represents the control humidity register (0x72) of the BME688 sensor.
    #[derive(Clone, Copy, PartialEq, Eq)]
    pub(crate) struct CtrlHum(u8);
    impl Debug;
    /// Oversampling of humidity (bits 2–0)
    pub u8, from into Oversampling, hum_os, set_hum_os: 2, 0;
}

bitfield::bitfield! {
    /// Represents the control gas registers (0x70–0x71) of the BME688 sensor.
    #[derive(Clone, Copy, PartialEq, Eq)]
    pub(crate) struct CtrlGas(u16);
    impl Debug;
    /// Disable standby time 0x71 (bit 7)
    pub bool, disable_standby, set_disable_standby: 15;
    /// Run gas measurement 0x71 (bit 5)
    pub bool, run_gas, set_run_gas: 13;
    /// Heater profile selection 0x71 (bits 3–0)
    pub u8, heater_profile, set_heater_profile: 11, 8;
    /// Disable heater 0x70 (bit 3)
    pub bool, disable_heater, set_disable_heater: 3;
}

/// Pressure ADC value represented as a 20-bit unsigned integer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct PressureAdc(pub u32);

impl From<u32> for PressureAdc {
    fn from(value: u32) -> Self {
        // Convert the value to big-endian format and shift the value
        // to the right by 12 bits to get the 20-bit ADC value
        PressureAdc(u32::from_be(value) >> 12)
    }
}

/// Temperature ADC value represented as a 20-bit unsigned integer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct TemperatureAdc(pub u32);

impl From<u32> for TemperatureAdc {
    fn from(value: u32) -> Self {
        // Convert the value to big-endian format and shift the value
        // to the right by 12 bits to get the 20-bit ADC value
        TemperatureAdc(u32::from_be(value) >> 12)
    }
}

/// Humidity ADC value represented as a 16-bit unsigned integer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct HumidityAdc(pub u16);

impl From<u16> for HumidityAdc {
    fn from(value: u16) -> Self {
        // Convert the value to big-endian format and store it as a 16-bit ADC value
        HumidityAdc(u16::from_be(value))
    }
}

/// Gas resistance ADC value represented as a 10-bit unsigned integer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct GasResistanceAdc(pub u16);

impl From<u16> for GasResistanceAdc {
    fn from(value: u16) -> Self {
        // Convert the value to big-endian format and shift the value
        // to the right by 6 bits to get the 10-bit ADC value
        GasResistanceAdc(u16::from_be(value) >> 6)
    }
}

bitfield::bitfield! {
    #[derive(Copy, Clone, PartialEq, Eq)]
    pub(crate) struct FieldData([u8]);
    impl Debug;
    /// New data flag (bit 7)
    pub bool, new_data, _: offset(0, 7);
    /// Gas measuring flag (bit 6)
    pub bool, gas_measuring, _: offset(0, 6);
    /// TPHG measuring flag (bit 5)
    pub bool, measuring, _: offset(0, 5);
    /// Gas measurement index (bits 3–0)
    pub u8, gas_meas_index, _: offset(0, 3), offset(0, 0);
    /// TPHG measurement index (bits 7–0)
    pub u8, meas_index, _: offset(1, 7), offset(1, 0);
    /// Pressure MSB (bits 7–0), LSB (bits 7–0), and XLSB (bits 7–4)
    pub u32, from into PressureAdc, press_adc, _: offset(4, 7), offset(2, 0);
    /// Temperature MSB (bits 7–0), LSB (bits 7–0), and XLSB (bits 7–4)
    pub u32, from into TemperatureAdc, temp_adc, _: offset(7, 7), offset(5, 0);
    /// Humidity MSB (bits 7–0) and LSB (bits 7–0)
    pub u16, from into HumidityAdc, hum_adc, _: offset(9, 7), offset(8, 0);
    /// Gas resistance MSB (bits 7–0) and LSB (bits 7–6)
    pub u16, from into GasResistanceAdc, gas_res_adc, _: offset(16, 7), offset(15, 0);
    /// Gas measurement valid flag (bit 5)
    pub bool, gas_valid, _: offset(16, 5);
    /// Gas measurement stable flag (bit 4)
    pub bool, heater_stable, _: offset(16, 4);
    /// Gas range (bits 3–0)
    pub u8, gas_range, _: offset(16, 3), offset(16, 0);
}

/// Read a sequence of registers from the BME688 sensor over I2C.
///
/// # Arguments
/// * `i2c` - The I2C interface to communicate with the sensor.
/// * `address` - The I2C address of the sensor.
/// * `start_register` - The starting register address to read from.
/// * `buffer` - The buffer to store the read data.
///
/// # Returns
/// A `Result` indicating success or failure of the read operation.
async fn read_registers<I2C>(
    i2c: &mut I2C,
    address: u8,
    start_register: u8,
    buffer: &mut [u8],
) -> Result<(), I2C::Error>
where
    I2C: embedded_hal_async::i2c::I2c,
{
    i2c.write_read(address, &[start_register], buffer).await
}

async fn read_register<I2C>(i2c: &mut I2C, address: u8, register: u8) -> Result<u8, I2C::Error>
where
    I2C: embedded_hal_async::i2c::I2c,
{
    let mut buffer = [0u8; 1];
    read_registers(i2c, address, register, &mut buffer).await?;
    Ok(buffer[0])
}

/// Write a value to a specific register of the BME688 sensor over I2C.
///
/// # Arguments
/// * `i2c` - The I2C interface to communicate with the sensor.
/// * `address` - The I2C address of the sensor.
/// * `register` - The register address to write to.
/// * `value` - The value to write to the register.
///
/// # Returns
/// A `Result` indicating success or failure of the write operation.
async fn write_register<I2C>(
    i2c: &mut I2C,
    address: u8,
    register: u8,
    value: u8,
) -> Result<(), I2C::Error>
where
    I2C: embedded_hal_async::i2c::I2c,
{
    i2c.write(address, &[register, value]).await
}

/// Perform a soft reset of the BME688 sensor.
///
/// # Arguments
/// * `i2c` - The I2C interface to communicate with the sensor.
/// * `address` - The I2C address of the sensor.
///
/// # Returns
/// A `Result` indicating success or failure of the soft reset operation.
pub(crate) async fn soft_reset<I2C>(i2c: &mut I2C, address: u8) -> Result<(), I2C::Error>
where
    I2C: embedded_hal_async::i2c::I2c,
{
    const RESET_COMMAND: u8 = 0xB6;
    write_register(i2c, address, REG_SOFT_RESET, RESET_COMMAND).await
}

/// Read the variant ID of the BME688 sensor.
///
/// # Arguments
/// * `i2c` - The I2C interface to communicate with the sensor.
/// * `address` - The I2C address of the sensor.
///
/// # Returns
/// A `Result` containing the variant ID or an error if the read operation fails.
pub(crate) async fn read_variant_id<I2C>(i2c: &mut I2C, address: u8) -> Result<u8, I2C::Error>
where
    I2C: embedded_hal_async::i2c::I2c,
{
    read_register(i2c, address, REG_VARIANT_ID).await
}

/// Read the chip ID of the BME688 sensor.
///
/// # Arguments
/// * `i2c` - The I2C interface to communicate with the sensor.
/// * `address` - The I2C address of the sensor.
///
/// # Returns
/// A `Result` containing the chip ID or an error if the read operation fails.
pub(crate) async fn read_chip_id<I2C>(i2c: &mut I2C, address: u8) -> Result<u8, I2C::Error>
where
    I2C: embedded_hal_async::i2c::I2c,
{
    read_register(i2c, address, REG_CHIP_ID).await
}

/// Read the calibration coefficients from the BME688 sensor.
///
/// # Arguments
/// * `i2c` - The I2C interface to communicate with the sensor.
/// * `address` - The I2C address of the sensor.
///
/// # Returns
/// A `Result` containing the calibration coefficients or an error if the read operation fails.
pub(crate) async fn read_coefficients<I2C>(
    i2c: &mut I2C,
    address: u8,
) -> Result<Coeffs<[u8; LEN_COEFF]>, I2C::Error>
where
    I2C: embedded_hal_async::i2c::I2c,
{
    let mut coeffs = Coeffs::<[u8; LEN_COEFF]>([0u8; LEN_COEFF]);

    // Read the first block of coefficients
    read_registers(i2c, address, REG_COEFF_1, &mut coeffs.0[0..LEN_COEFF_1]).await?;

    // Read the second block of coefficients
    read_registers(
        i2c,
        address,
        REG_COEFF_2,
        &mut coeffs.0[LEN_COEFF_1..(LEN_COEFF_1 + LEN_COEFF_2)],
    )
    .await?;

    // Read the third block of coefficients
    read_registers(
        i2c,
        address,
        REG_COEFF_3,
        &mut coeffs.0[(LEN_COEFF_1 + LEN_COEFF_2)..],
    )
    .await?;

    Ok(coeffs.into())
}

/// Read the configuration register from the BME688 sensor.
///
/// # Arguments
/// * `i2c` - The I2C interface to communicate with the sensor.
/// * `address` - The I2C address of the sensor.
///
/// # Returns
/// A `Result` containing the configuration register or an error if the read operation fails.
pub(crate) async fn read_config<I2C>(i2c: &mut I2C, address: u8) -> Result<Config, I2C::Error>
where
    I2C: embedded_hal_async::i2c::I2c,
{
    let config = read_register(i2c, address, REG_CONFIG).await?;
    Ok(Config(config))
}

/// Write the configuration register to the BME688 sensor.
///
/// # Arguments
/// * `i2c` - The I2C interface to communicate with the sensor.
/// * `address` - The I2C address of the sensor.
/// * `config` - The configuration register to write.
///
/// # Returns
/// A `Result` indicating success or failure of the write operation.
pub(crate) async fn write_config<I2C>(
    i2c: &mut I2C,
    address: u8,
    config: Config,
) -> Result<(), I2C::Error>
where
    I2C: embedded_hal_async::i2c::I2c,
{
    write_register(i2c, address, REG_CONFIG, config.0).await
}

/// Read the control measurement register from the BME688 sensor.
///
/// # Arguments
/// * `i2c` - The I2C interface to communicate with the sensor.
/// * `address` - The I2C address of the sensor.
///
/// # Returns
/// A `Result` containing the control measurement register or an error if the read operation fails.
pub(crate) async fn read_ctrl_meas<I2C>(i2c: &mut I2C, address: u8) -> Result<CtrlMeas, I2C::Error>
where
    I2C: embedded_hal_async::i2c::I2c,
{
    let ctrl_meas = read_register(i2c, address, REG_CTRL_MEAS).await?;
    Ok(CtrlMeas(ctrl_meas))
}

/// Write the control measurement register to the BME688 sensor.
///
/// # Arguments
/// * `i2c` - The I2C interface to communicate with the sensor.
/// * `address` - The I2C address of the sensor.
/// * `ctrl_meas` - The control measurement register to write.
///
/// # Returns
/// A `Result` indicating success or failure of the write operation.
pub(crate) async fn write_ctrl_meas<I2C>(
    i2c: &mut I2C,
    address: u8,
    ctrl_meas: CtrlMeas,
) -> Result<(), I2C::Error>
where
    I2C: embedded_hal_async::i2c::I2c,
{
    write_register(i2c, address, REG_CTRL_MEAS, ctrl_meas.0).await
}

/// Read the control humidity register from the BME688 sensor.
///
/// # Arguments
/// * `i2c` - The I2C interface to communicate with the sensor.
/// * `address` - The I2C address of the sensor.
///
/// # Returns
/// A `Result` containing the control humidity register or an error if the read operation fails.
pub(crate) async fn read_ctrl_hum<I2C>(i2c: &mut I2C, address: u8) -> Result<CtrlHum, I2C::Error>
where
    I2C: embedded_hal_async::i2c::I2c,
{
    let ctrl_hum = read_register(i2c, address, REG_CTRL_HUM).await?;
    Ok(CtrlHum(ctrl_hum))
}

/// Write the control humidity register to the BME688 sensor.
///
/// # Arguments
/// * `i2c` - The I2C interface to communicate with the sensor.
/// * `address` - The I2C address of the sensor.
/// * `ctrl_hum` - The control humidity register to write.
///
/// # Returns
/// A `Result` indicating success or failure of the write operation.
pub(crate) async fn write_ctrl_hum<I2C>(
    i2c: &mut I2C,
    address: u8,
    ctrl_hum: CtrlHum,
) -> Result<(), I2C::Error>
where
    I2C: embedded_hal_async::i2c::I2c,
{
    write_register(i2c, address, REG_CTRL_HUM, ctrl_hum.0).await
}

/// Read the control gas registers from the BME688 sensor.
///
/// # Arguments
/// * `i2c` - The I2C interface to communicate with the sensor.
/// * `address` - The I2C address of the sensor.
///
/// # Returns
/// A `Result` containing the control gas registers or an error if the read operation fails.
pub(crate) async fn read_ctrl_gas<I2C>(i2c: &mut I2C, address: u8) -> Result<CtrlGas, I2C::Error>
where
    I2C: embedded_hal_async::i2c::I2c,
{
    let mut buffer = [0u8; LEN_CTRL_GAS];
    read_registers(i2c, address, REG_CTRL_GAS_0, &mut buffer).await?;
    let ctrl_gas = u16::from_le_bytes(buffer);
    Ok(CtrlGas(ctrl_gas))
}

/// Write the control gas registers to the BME688 sensor.
///
/// # Arguments
/// * `i2c` - The I2C interface to communicate with the sensor.
/// * `address` - The I2C address of the sensor.
/// * `ctrl_gas` - The control gas registers to write.
///
/// # Returns
/// A `Result` indicating success or failure of the write operation.
pub(crate) async fn write_ctrl_gas<I2C>(
    i2c: &mut I2C,
    address: u8,
    ctrl_gas: CtrlGas,
) -> Result<(), I2C::Error>
where
    I2C: embedded_hal_async::i2c::I2c,
{
    let [lsb, msb] = ctrl_gas.0.to_le_bytes();
    write_register(i2c, address, REG_CTRL_GAS_0, lsb).await?;
    write_register(i2c, address, REG_CTRL_GAS_1, msb).await?;
    Ok(())
}

/// Read the field data from the BME688 sensor.
///
/// # Arguments
/// * `i2c` - The I2C interface to communicate with the sensor.
/// * `address` - The I2C address of the sensor.
/// * `index` - The index of the field data to read (0, 1, or 2).
///
/// # Returns
/// A `Result` containing the field data or an error if the read operation fails.
pub(crate) async fn read_field_data<I2C>(
    i2c: &mut I2C,
    address: u8,
    index: u8,
) -> Result<FieldData<[u8; LEN_FIELD]>, I2C::Error>
where
    I2C: embedded_hal_async::i2c::I2c,
{
    assert!(
        index < MAX_FIELD_COUNT as u8,
        "Index must be less than {}",
        MAX_FIELD_COUNT
    );

    let mut field_data = FieldData::<[u8; LEN_FIELD]>([0u8; LEN_FIELD]);
    read_registers(
        i2c,
        address,
        REG_FIELD_0 + index * LEN_FIELD as u8,
        &mut field_data.0,
    )
    .await?;
    Ok(field_data)
}

/// Read the gas wait duration for a specific heater profile from the BME688 sensor.
///
/// # Arguments
/// * `i2c` - The I2C interface to communicate with the sensor.
/// * `address` - The I2C address of the sensor.
/// * `index` - The index of the heater profile to read (0 to 9).
///
/// # Returns
/// A `Result` containing the gas wait value or an error if the read operation fails.
pub(crate) async fn read_gas_wait<I2C>(
    i2c: &mut I2C,
    address: u8,
    index: u8,
) -> Result<u8, I2C::Error>
where
    I2C: embedded_hal_async::i2c::I2c,
{
    assert!(
        index < MAX_HEATER_PROFILES as u8,
        "Index must be less than {}",
        MAX_HEATER_PROFILES
    );

    read_register(i2c, address, REG_GAS_WAIT_0 + index).await
}

/// Write the gas wait duration for a specific heater profile to the BME688 sensor.
///
/// # Arguments
/// * `i2c` - The I2C interface to communicate with the sensor.
/// * `address` - The I2C address of the sensor.
/// * `index` - The index of the heater profile to write (0 to 9).
/// * `value` - The value to write to the gas wait register.
///
/// # Returns
/// A `Result` indicating success or failure of the write operation.
pub(crate) async fn write_gas_wait<I2C>(
    i2c: &mut I2C,
    address: u8,
    index: u8,
    value: u8,
) -> Result<(), I2C::Error>
where
    I2C: embedded_hal_async::i2c::I2c,
{
    assert!(
        index < MAX_HEATER_PROFILES as u8,
        "Index must be less than {}",
        MAX_HEATER_PROFILES
    );

    write_register(i2c, address, REG_GAS_WAIT_0 + index, value).await
}

/// Read the IDAC heat value for a specific heater profile from the BME688 sensor.
///
/// # Arguments
/// * `i2c` - The I2C interface to communicate with the sensor.
/// * `address` - The I2C address of the sensor.
/// * `index` - The index of the heater profile to read (0 to 9).
///
/// # Returns
/// A `Result` containing the IDAC heat value or an error if the read operation fails.
pub(crate) async fn read_idac_heat<I2C>(
    i2c: &mut I2C,
    address: u8,
    index: u8,
) -> Result<u8, I2C::Error>
where
    I2C: embedded_hal_async::i2c::I2c,
{
    assert!(
        index < MAX_HEATER_PROFILES as u8,
        "Index must be less than {}",
        MAX_HEATER_PROFILES
    );

    read_register(i2c, address, REG_IDAC_HEAT_0 + index).await
}

/// Read the heater resistance value for a specific heater profile from the BME688 sensor.
///
/// # Arguments
/// * `i2c` - The I2C interface to communicate with the sensor.
/// * `address` - The I2C address of the sensor.
/// * `index` - The index of the heater profile to read (0 to 9).
///
/// # Returns
/// A `Result` containing the resistance heat value or an error if the read operation fails.
pub(crate) async fn read_res_heat<I2C>(
    i2c: &mut I2C,
    address: u8,
    index: u8,
) -> Result<u8, I2C::Error>
where
    I2C: embedded_hal_async::i2c::I2c,
{
    assert!(
        index < MAX_HEATER_PROFILES as u8,
        "Index must be less than {}",
        MAX_HEATER_PROFILES
    );

    read_register(i2c, address, REG_RES_HEAT_0 + index).await
}

/// Write the heater resistance value for a specific heater profile to the BME688 sensor.
///
/// # Arguments
/// * `i2c` - The I2C interface to communicate with the sensor.
/// * `address` - The I2C address of the sensor.
/// * `index` - The index of the heater profile to write (0 to 9).
/// * `value` - The value to write to the resistance heater register.
///
/// # Returns
/// A `Result` indicating success or failure of the write operation.
pub(crate) async fn write_res_heat<I2C>(
    i2c: &mut I2C,
    address: u8,
    index: u8,
    value: u8,
) -> Result<(), I2C::Error>
where
    I2C: embedded_hal_async::i2c::I2c,
{
    assert!(
        index < MAX_HEATER_PROFILES as u8,
        "Index must be less than {}",
        MAX_HEATER_PROFILES
    );

    write_register(i2c, address, REG_RES_HEAT_0 + index, value).await
}

/// Read all field data from the BME688 sensor.
///
/// # Arguments
/// * `i2c` - The I2C interface to communicate with the sensor.
/// * `address` - The I2C address of the sensor.
///
///# Returns
/// A `Result` containing an array of field data or an error if the read operation fails.
pub(crate) async fn read_all_field_data<I2C>(
    i2c: &mut I2C,
    address: u8,
) -> Result<[FieldData<[u8; LEN_FIELD]>; MAX_FIELD_COUNT], I2C::Error>
where
    I2C: embedded_hal_async::i2c::I2c,
{
    let mut buffer = [0u8; LEN_FIELD * MAX_FIELD_COUNT];
    registers::read_registers(i2c, address, REG_FIELD_0, &mut buffer).await?;

    let mut chunks = buffer.chunks_exact(LEN_FIELD);
    let all_field_data = core::array::from_fn(|_| {
        let chunk: [u8; LEN_FIELD] = chunks.next().unwrap().try_into().unwrap();
        FieldData(chunk)
    });

    Ok(all_field_data)
}

/// Write the shared gas wait duration to the BME688 sensor.
///
/// # Arguments
/// * `i2c` - The I2C interface to communicate with the sensor.
/// * `address` - The I2C address of the sensor.
/// * `value` - The value to write to the shared gas wait register.
///
/// # Returns
/// A `Result` indicating success or failure of the write operation.
pub(crate) async fn write_shared_gas_wait<I2C>(
    i2c: &mut I2C,
    address: u8,
    value: u8,
) -> Result<(), I2C::Error>
where
    I2C: embedded_hal_async::i2c::I2c,
{
    write_register(i2c, address, REG_SHD_GAS_WAIT, value).await
}

#[cfg(test)]
mod tests {

}