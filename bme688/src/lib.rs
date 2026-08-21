#![cfg_attr(not(test), no_std)]

mod error;
pub use error::Error;
mod math;
mod registers;
pub use registers::MAX_FIELD_COUNT as MAX_MEASUREMENTS;
pub use registers::{Filter, Mode, Oversampling, StandbyTime};
mod config;
pub use config::{forced, parallel, sequential};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SlaveAddress {
    /// Primary slave address (0x76)
    #[default]
    Primary,
    /// Secondary slave address (0x77)
    Secondary,
    /// Other slave address (0x00 - 0x7F)
    Other(u8),
}

impl From<SlaveAddress> for u8 {
    fn from(address: SlaveAddress) -> Self {
        match address {
            SlaveAddress::Primary => 0x76,
            SlaveAddress::Secondary => 0x77,
            SlaveAddress::Other(value) => value,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
struct Coeffs {
    /// Temperature compensation coefficient T1
    t1: u16,
    /// Temperature compensation coefficient T2
    t2: i16,
    /// Temperature compensation coefficient T3
    t3: i8,
    /// Pressure compensation coefficient P1
    p1: u16,
    /// Pressure compensation coefficient P2
    p2: i16,
    /// Pressure compensation coefficient P3
    p3: i8,
    /// Pressure compensation coefficient P4
    p4: i16,
    /// Pressure compensation coefficient P5
    p5: i16,
    /// Pressure compensation coefficient P6
    p6: i8,
    /// Pressure compensation coefficient P7
    p7: i8,
    /// Pressure compensation coefficient P8
    p8: i16,
    /// Pressure compensation coefficient P9
    p9: i16,
    /// Pressure compensation coefficient P10
    p10: u8,
    /// Humidity compensation coefficient H1
    h1: u16,
    /// Humidity compensation coefficient H2
    h2: u16,
    /// Humidity compensation coefficient H3
    h3: i8,
    /// Humidity compensation coefficient H4
    h4: i8,
    /// Humidity compensation coefficient H5
    h5: i8,
    /// Humidity compensation coefficient H6
    h6: u8,
    /// Humidity compensation coefficient H7
    h7: i8,
    /// Gas resistance compensation coefficient G1
    g1: i8,
    /// Gas resistance compensation coefficient G2
    g2: i16,
    /// Gas resistance compensation coefficient G3
    g3: i8,
    /// Heater resistance value
    res_heat_val: i8,
    /// Heater resistance range
    res_heat_range: u8,
    /// Range switching error
    range_sw_error: i8,
}

impl From<registers::Coeffs<[u8; registers::LEN_COEFF]>> for Coeffs {
    fn from(raw: registers::Coeffs<[u8; registers::LEN_COEFF]>) -> Self {
        Coeffs {
            t1: raw.t1(),
            t2: raw.t2(),
            t3: raw.t3(),
            p1: raw.p1(),
            p2: raw.p2(),
            p3: raw.p3(),
            p4: raw.p4(),
            p5: raw.p5(),
            p6: raw.p6(),
            p7: raw.p7(),
            p8: raw.p8(),
            p9: raw.p9(),
            p10: raw.p10(),
            h1: raw.h1().0,
            h2: raw.h2().0,
            h3: raw.h3(),
            h4: raw.h4(),
            h5: raw.h5(),
            h6: raw.h6(),
            h7: raw.h7(),
            g1: raw.g1(),
            g2: raw.g2(),
            g3: raw.g3(),
            res_heat_val: raw.res_heat_val(),
            res_heat_range: raw.res_heat_range(),
            range_sw_error: raw.range_sw_error(),
        }
    }
}

bitfield::bitfield! {
    /// Represents the status of the sensor, including flags for new data, measuring state, gas measurement state, and heater stability.
    #[derive(Clone, Copy, PartialEq, Eq)]
    pub struct Status(u8);
    impl Debug;
    /// Indicates if new data is available from the sensor.
    pub is_new_data, set_is_new_data: 7;
    /// Indicates if the sensor is currently measuring temperature, pressure, humidity, or gas resistance.
    pub is_measuring, set_is_measuring: 6;
    /// Indicates if the sensor is currently measuring gas resistance.
    pub is_gas_measuring, set_is_gas_measuring: 5;
    /// Indicates if the gas measurement is valid (i.e., the heater has reached the target temperature).
    pub is_gas_valid, set_is_gas_valid: 4;
    /// Indicates if the heater is stable (i.e., the heater has reached the target temperature and is maintaining it).
    pub is_heater_stable, set_is_heater_stable: 3;
}

#[derive(Debug, Clone, Copy)]
pub struct Measurement {
    /// Status flags indicating the state of the sensor and measurements.
    pub status: Status,
    /// Index of the temperature, pressure, humidity, and gas measurement.
    pub meas_index: u8,
    /// Index of the gas measurement.
    pub gas_meas_index: u8,
    /// The temperature reading in degrees Celsius.
    pub temperature: f32,
    /// The pressure reading in Pascals.
    pub pressure: f32,
    /// The humidity reading in percentage relative humidity x1000.
    pub humidity: f32,
    /// The gas resistance reading in Ohms.
    pub gas_resistance: f32,
}

pub struct Uninit;
pub struct Init;

mod sealed {
    pub trait Sealed {}
    impl Sealed for super::Uninit {}
    impl Sealed for super::Init {}
}

pub struct Bme688<I2C, D, State>
where
    I2C: embedded_hal_async::i2c::I2c,
    D: embedded_hal_async::delay::DelayNs,
    State: sealed::Sealed,
{
    /// The I2C interface to use for communication with the sensor.
    i2c: I2C,
    /// The I2C address of the sensor.
    address: u8,
    /// The delay provider for timing operations.
    delay: D,
    /// The compensation coefficients for the sensor.
    coeffs: Coeffs,
    /// The current state of the sensor (Uninit, Sleep, Forced, Parallel, or Sequential).
    state: core::marker::PhantomData<State>,
}

impl<I2C, D> Bme688<I2C, D, Uninit>
where
    I2C: embedded_hal_async::i2c::I2c,
    D: embedded_hal_async::delay::DelayNs,
{
    /// Expected chip ID for the BME688 sensor.
    const CHIP_ID: u8 = 0x61;
    /// Expected variant ID for the BME688 sensor.
    const VARIANT_ID: u8 = 0x01;
    /// Delay in microseconds after a soft reset to allow the sensor to initialize.
    const RESET_DELAY_US: u32 = 5000;

    pub fn new(i2c: I2C, address: SlaveAddress, delay: D) -> Self {
        Bme688 {
            i2c,
            address: address.into(),
            delay,
            coeffs: Coeffs::default(),
            state: core::marker::PhantomData,
        }
    }

    pub async fn init(mut self) -> Result<Bme688<I2C, D, Init>, crate::Error<I2C::Error>> {
        // Perform a soft reset of the sensor to ensure it is in a known state
        registers::soft_reset(&mut self.i2c, self.address).await?;
        self.delay.delay_us(Self::RESET_DELAY_US).await;

        // Read the chip ID to verify that the sensor is present
        let chip_id = registers::read_chip_id(&mut self.i2c, self.address).await?;
        if chip_id != Self::CHIP_ID {
            return Err(crate::Error::UnexpectedChipId(chip_id));
        }

        // Read the variant ID to verify that the sensor is the expected variant
        let variant_id = registers::read_variant_id(&mut self.i2c, self.address).await?;
        if variant_id != Self::VARIANT_ID {
            return Err(crate::Error::UnexpectedVariantId(variant_id));
        }

        // Read the compensation coefficients from the sensor
        let coeffs = registers::read_coefficients(&mut self.i2c, self.address).await?;

        Ok(Bme688 {
            i2c: self.i2c,
            address: self.address,
            delay: self.delay,
            coeffs: coeffs.into(),
            state: core::marker::PhantomData,
        })
    }
}

impl<I2C, D> Bme688<I2C, D, Init>
where
    I2C: embedded_hal_async::i2c::I2c,
    D: embedded_hal_async::delay::DelayNs,
{
    /// The maximum number of attempts to set the sensor to sleep mode before timing out.
    const SLEEP_MODE_ATTEMPTS: u8 = 5;
    /// The delay in microseconds between attempts to set the sensor to sleep mode.
    const POLL_DELAY_US: u32 = 10000;

    /// Sets the operating mode of the sensor.
    ///
    /// # Arguments
    /// * `mode` - The desired operating mode (Sleep, Forced, Parallel or Sequential).
    /// # Returns
    /// A result indicating success or an error if the operation fails.
    async fn set_mode(&mut self, mode: Mode) -> Result<(), Error<I2C::Error>> {
        for _ in 0..Self::SLEEP_MODE_ATTEMPTS {
            let mut ctrl_meas = registers::read_ctrl_meas(&mut self.i2c, self.address).await?;
            if ctrl_meas.mode() == Mode::Sleep {
                ctrl_meas.set_mode(mode);
                registers::write_ctrl_meas(&mut self.i2c, self.address, ctrl_meas).await?;
                return Ok(());
            }
            ctrl_meas.set_mode(Mode::Sleep);
            registers::write_ctrl_meas(&mut self.i2c, self.address, ctrl_meas).await?;
            self.delay.delay_us(Self::POLL_DELAY_US).await;
        }
        Err(Error::SleepModeTimeout)
    }

    async fn set_config(
        &mut self,
        config: &config::Config<impl config::sealed::Sealed>,
    ) -> Result<(), Error<I2C::Error>> {
        // Set temperature, pressure, and humidity oversampling
        let mut ctrl_meas = registers::read_ctrl_meas(&mut self.i2c, self.address).await?;
        ctrl_meas.set_temp_os(config.temperature_oversampling);
        ctrl_meas.set_press_os(config.pressure_oversampling);
        registers::write_ctrl_meas(&mut self.i2c, self.address, ctrl_meas).await?;

        let mut ctrl_hum = registers::read_ctrl_hum(&mut self.i2c, self.address).await?;
        ctrl_hum.set_hum_os(config.humidity_oversampling);
        registers::write_ctrl_hum(&mut self.i2c, self.address, ctrl_hum).await?;

        // Set IIR filter for pressure and temperature
        let mut config_reg = registers::read_config(&mut self.i2c, self.address).await?;
        config_reg.set_filter(config.filter);
        if let Some(standby_time) = config.standby_time {
            config_reg.set_standby_time(standby_time);
        }
        registers::write_config(&mut self.i2c, self.address, config_reg).await?;

        // Set heater resistance and gas wait time for each heater profile
        for (index, heater_step) in config.heater_profile.iter().enumerate() {
            let res_heat = math::calc_heater_resistance(
                heater_step.target_temp,
                config.ambient_temperature,
                &self.coeffs,
            );
            registers::write_res_heat(&mut self.i2c, self.address, index as u8, res_heat).await?;
            registers::write_gas_wait(
                &mut self.i2c,
                self.address,
                index as u8,
                heater_step.gas_wait,
            )
            .await?;
        }

        Ok(())
    }

    /// Starts a forced measurement with the specified configuration.
    ///
    /// The heater is on after TPH measurement until gas measurement is complete.
    /// 
    /// # Arguments
    /// * `config` - The configuration for the forced measurement.
    ///
    /// # Returns
    /// A result containing the TPHG measurement duration in microseconds (without heater on time) or an error if the operation fails.
    pub async fn start_forced_measurement(
        &mut self,
        config: &config::forced::Config,
    ) -> Result<u32, Error<I2C::Error>> {
        // Set the sensor to sleep mode before configuring it
        self.set_mode(Mode::Sleep).await?;

        // Apply the configuration settings to the sensor
        self.set_config(config).await?;

        if config.heater_profile.is_empty() {
            // Disable gas conversion if no gas configuration is provided
            let mut ctrl_gas = registers::read_ctrl_gas(&mut self.i2c, self.address).await?;
            ctrl_gas.set_run_gas(false);
            ctrl_gas.set_disable_heater(true);
            registers::write_ctrl_gas(&mut self.i2c, self.address, ctrl_gas).await?;
        } else {
            // Enable gas conversion and set the first heater profile
            let mut ctrl_gas = registers::read_ctrl_gas(&mut self.i2c, self.address).await?;
            ctrl_gas.set_run_gas(true);
            ctrl_gas.set_disable_heater(false);
            ctrl_gas.set_heater_profile(0);
            registers::write_ctrl_gas(&mut self.i2c, self.address, ctrl_gas).await?;
        }
        self.set_mode(Mode::Forced).await?;

        Ok(config.measurement_duration(Mode::Forced))
    }

    /// Starts a parallel measurement with the specified configuration.
    /// 
    /// The heater is on for the entire duration of the measurement, and the gas wait time is shared across all heater profiles.
    /// 
    /// # Arguments
    /// * `config` - The configuration for the parallel measurement.
    ///
    /// # Returns
    /// A result containing the TPHG measurement duration in microseconds (without the shared gas wait time) or an error if the operation fails.
    pub async fn start_parallel_measurement(
        &mut self,
        config: &config::parallel::Config,
    ) -> Result<u32, crate::Error<I2C::Error>> {
        // Set the sensor to sleep mode before configuring it
        self.set_mode(Mode::Sleep).await?;

        // Apply the configuration settings to the sensor
        self.set_config(config).await?;

        if config.heater_profile.is_empty() {
            // Disable gas conversion if no gas configuration is provided
            let mut ctrl_gas = registers::read_ctrl_gas(&mut self.i2c, self.address).await?;
            ctrl_gas.set_run_gas(false);
            ctrl_gas.set_disable_heater(true);
            registers::write_ctrl_gas(&mut self.i2c, self.address, ctrl_gas).await?;
        } else {
            let shared_gas_wait_duration_ms = config
                .shared_heater_duration_ms
                .saturating_sub(config.measurement_duration(Mode::Parallel) * 1000) as u16;

            // Set the shared gas wait duration for parallel mode
            registers::write_shared_gas_wait(
                &mut self.i2c,
                self.address,
                math::calc_shared_gas_wait_duration(shared_gas_wait_duration_ms),
            )
            .await?;

            // Enable gas conversion and set the first heater profile
            let mut ctrl_gas = registers::read_ctrl_gas(&mut self.i2c, self.address).await?;
            ctrl_gas.set_run_gas(true);
            ctrl_gas.set_disable_heater(false);
            ctrl_gas.set_heater_profile(config.heater_profile.len() as u8);
            registers::write_ctrl_gas(&mut self.i2c, self.address, ctrl_gas).await?;
        };
        self.set_mode(Mode::Parallel).await?;

        Ok(config.measurement_duration(Mode::Parallel))
    }

    /// Starts a sequential measurement with the specified configuration.
    ///
    /// # Arguments
    /// * `config` - The configuration for the sequential measurement.
    ///
    /// # Returns
    /// A result containing the TPHG measurement duration in microseconds (without heater on time) or an error if the operation fails.
    pub async fn start_sequential_measurement(
        &mut self,
        config: &config::sequential::Config,
    ) -> Result<u32, crate::Error<I2C::Error>> {
        // Set the sensor to sleep mode before configuring it
        self.set_mode(Mode::Sleep).await?;

        // Apply the configuration settings to the sensor
        self.set_config(config).await?;

        if config.heater_profile.is_empty() {
            // Disable gas conversion if no gas configuration is provided
            let mut ctrl_gas = registers::read_ctrl_gas(&mut self.i2c, self.address).await?;
            ctrl_gas.set_run_gas(false);
            ctrl_gas.set_disable_heater(true);
            registers::write_ctrl_gas(&mut self.i2c, self.address, ctrl_gas).await?;
        } else {
            // Enable gas conversion and set the first heater profile
            let mut ctrl_gas = registers::read_ctrl_gas(&mut self.i2c, self.address).await?;
            ctrl_gas.set_run_gas(true); 
            ctrl_gas.set_disable_heater(false);
            ctrl_gas.set_heater_profile(config.heater_profile.len() as u8);
            ctrl_gas.set_disable_standby(config.standby_time.is_none());
            registers::write_ctrl_gas(&mut self.i2c, self.address, ctrl_gas).await?;
        }
        self.set_mode(Mode::Sequential).await?;

        Ok(config.measurement_duration(Mode::Sequential))
    }

    pub async fn sleep(&mut self) -> Result<(), Error<I2C::Error>> {
        self.set_mode(Mode::Sleep).await
    }

    /// Decodes the raw measurement data from the sensor into a `Measurement` struct.
    ///
    /// # Arguments
    /// * `field_data` - The raw field data read from the sensor.
    ///
    /// # Returns
    /// A `Measurement` struct containing the decoded sensor readings.
    fn decode_measurement(
        &mut self,
        field_data: registers::FieldData<[u8; registers::LEN_FIELD]>,
    ) -> Measurement {
        let mut status = Status(0);
        status.set_is_new_data(field_data.new_data());
        status.set_is_measuring(field_data.measuring());
        status.set_is_gas_measuring(field_data.gas_measuring());
        status.set_is_gas_valid(field_data.gas_valid());
        status.set_is_heater_stable(field_data.heater_stable());

        let (temperature, temp_fine) =
            math::calc_temperature(field_data.temp_adc().0, &self.coeffs);
        let pressure = math::calc_pressure(field_data.press_adc().0, temp_fine, &self.coeffs);
        let humidity = math::calc_humidity(field_data.hum_adc().0, temp_fine, &self.coeffs);
        let gas_resistance =
            math::calc_gas_resistance(field_data.gas_res_adc().0, field_data.gas_range());

        Measurement {
            status,
            meas_index: field_data.meas_index(),
            gas_meas_index: field_data.gas_meas_index(),
            temperature,
            pressure,
            humidity,
            gas_resistance,
        }
    }

    /// Reads the measurement data from the sensor.
    ///
    /// # Returns
    /// A `Result` containing the `Measurement` struct on success, or an `Error` if the operation fails.
    pub async fn get_measurement(&mut self) -> Result<Measurement, Error<I2C::Error>> {
        let field_data = registers::read_field_data(&mut self.i2c, self.address, 0).await?;
        Ok(self.decode_measurement(field_data))
    }

    /// Reads all available measurements from the sensor.
    ///
    /// # Returns
    /// A `Result` containing a vector of `Measurement` structs on success, or an `Error` if the operation fails.
    pub async fn get_measurements(
        &mut self,
    ) -> Result<heapless::Vec<Measurement, { MAX_MEASUREMENTS }>, Error<I2C::Error>> {
        let field_data_array = registers::read_all_field_data(&mut self.i2c, self.address).await?;
        let mut measurements = field_data_array
            .into_iter()
            .filter_map(|field_data| {
                if field_data.new_data() {
                    Some(self.decode_measurement(field_data))
                } else {
                    None
                }
            })
            .collect::<heapless::Vec<Measurement, { MAX_MEASUREMENTS }>>();

        // meas_index wraps 255 -> 0, so sort by circular distance rather than raw value.
        measurements.sort_unstable_by(|a, b| {
            let diff = a.meas_index.wrapping_sub(b.meas_index) as i8;
            diff.cmp(&0)
        });

        Ok(measurements)
    }
}
