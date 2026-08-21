use crate::Mode;

pub struct Forced;
pub struct Parallel;
pub struct Sequential;

pub(crate) mod sealed {
    pub trait Sealed {}

    impl Sealed for super::Forced {}
    impl Sealed for super::Parallel {}
    impl Sealed for super::Sequential {}
}

pub(crate) struct HeaterStep {
    pub(crate) target_temp: u16,
    pub(crate) gas_wait: u8,
}

/// Generic configuration for the BME688 sensor.
pub struct Config<State>
where
    State: sealed::Sealed,
{
    /// Oversampling settings for temperature.
    pub(crate) temperature_oversampling: crate::Oversampling,
    /// Oversampling settings for pressure.
    pub(crate) pressure_oversampling: crate::Oversampling,
    /// Oversampling settings for humidity.
    pub(crate) humidity_oversampling: crate::Oversampling,
    /// IIR filter settings for pressure and temperature.
    pub(crate) filter: crate::Filter,
    /// Heater profile settings for the gas sensor.
    pub(crate) heater_profile: heapless::Vec<HeaterStep, { crate::registers::MAX_HEATER_PROFILES }>,
    /// Shared heater duration for parallel mode. This is the total duration for which the heater will be active during a measurement cycle.
    pub(crate) shared_heater_duration_ms: u32,
    /// Standby time for sequential mode. This is the duration for which the sensor will be in standby between measurements.
    pub(crate) standby_time: Option<crate::StandbyTime>,
    /// Ambient temperature
    pub(crate) ambient_temperature: i16,
    state: core::marker::PhantomData<State>,
}

impl<State> Default for Config<State>
where
    State: sealed::Sealed,
{
    fn default() -> Self {
        Self {
            temperature_oversampling: Default::default(),
            pressure_oversampling: Default::default(),
            humidity_oversampling: Default::default(),
            filter: Default::default(),
            heater_profile: heapless::Vec::new(),
            shared_heater_duration_ms: Default::default(),
            standby_time: Default::default(),
            ambient_temperature: 25,
            state: core::marker::PhantomData,
        }
    }
}

impl<State> Config<State>
where
    State: sealed::Sealed,
{
    const CYCLE_DURATION: u32 = 1963;
    const TPH_SWITCHING_DURATION: u32 = 477 * 4;
    const GAS_MEAS_DURATION: u32 = 477 * 5;
    const WAKEUP_DURATION: u32 = 1000;

    /// Calculate the total measurement duration based on the configuration and mode.
    ///
    /// # Arguments
    /// * `mode` - The mode of operation (Forced, Parallel, or Sequential).
    ///
    /// # Returns
    /// The total measurement duration in microseconds.
    pub fn measurement_duration(&self, mode: Mode) -> u32 {
        let cycles = self.temperature_oversampling.cycles()
            + self.pressure_oversampling.cycles()
            + self.humidity_oversampling.cycles();
        let duration_us = Self::CYCLE_DURATION * cycles as u32
            + Self::TPH_SWITCHING_DURATION
            + Self::GAS_MEAS_DURATION
            + if matches!(mode, Mode::Parallel) {
                0
            } else {
                Self::WAKEUP_DURATION
            };

        duration_us
    }
}

/// Builder pattern for constructing a `Config` instance.
pub struct ConfigBuilder<State>
where
    State: sealed::Sealed,
{
    config: Config<State>,
}

impl<State> ConfigBuilder<State>
where
    State: sealed::Sealed,
{
    /// Create a new `ConfigBuilder` with default settings.
    pub fn new() -> Self {
        Self {
            config: Config::default(),
        }
    }

    /// Set the oversampling for temperature measurements.
    ///
    /// # Arguments
    /// * `oversampling` - The desired oversampling setting for temperature.
    ///
    /// # Returns
    pub fn with_temperature_os(mut self, oversampling: crate::Oversampling) -> Self {
        self.config.temperature_oversampling = oversampling;
        self
    }
    /// Set the oversampling for pressure measurements.
    ///
    /// # Arguments
    /// * `oversampling` - The desired oversampling setting for pressure.
    ///
    /// # Returns
    pub fn with_pressure_os(mut self, oversampling: crate::Oversampling) -> Self {
        self.config.pressure_oversampling = oversampling;
        self
    }

    /// Set the oversampling for humidity measurements.
    ///
    /// # Arguments
    /// * `oversampling` - The desired oversampling setting for humidity.
    ///
    /// # Returns
    /// The updated `ConfigBuilder` instance.
    pub fn with_humidity_os(mut self, oversampling: crate::Oversampling) -> Self {
        self.config.humidity_oversampling = oversampling;
        self
    }

    /// Set the IIR filter settings for pressure and temperature.
    ///
    /// # Arguments
    /// * `filter` - The desired IIR filter setting.
    ///
    /// # Returns
    /// The updated `ConfigBuilder` instance.
    pub fn with_filter(mut self, filter: crate::Filter) -> Self {
        self.config.filter = filter;
        self
    }

    /// Set the ambient temperature.
    /// 
    /// # Arguments
    /// * `temperature` - The ambient temperature in degrees Celsius.
    ///
    /// # Returns
    /// The updated `ConfigBuilder` instance.
    pub fn with_ambient_temperature(mut self, temperature: i16) -> Self {
        self.config.ambient_temperature = temperature;
        self
    } 

    /// Finalize the builder and return the constructed `Config` instance.
    ///
    /// # Returns
    /// The constructed `Config` instance.
    pub fn build(self) -> Config<State> {
        self.config
    }
}

pub mod forced {
    pub type Config = super::Config<super::Forced>;
    pub type ConfigBuilder = super::ConfigBuilder<super::Forced>;

    impl super::ConfigBuilder<super::Forced> {
        /// Add a heater step to the configuration for forced mode.
        ///
        /// # Arguments
        /// * `target_temp` - The target temperature for the heater in degrees Celsius.
        /// * `duration_ms` - The duration for which the heater should be active in milliseconds.
        ///
        /// # Returns
        /// The updated `ConfigBuilder` instance.
        pub fn with_heater_step(mut self, target_temp: u16, duration_ms: u16) -> Self {
            let _ = self.config.heater_profile.insert(
                0,
                super::HeaterStep {
                    target_temp,
                    gas_wait: crate::math::calc_gas_wait_time(duration_ms),
                },
            );
            self
        }
    }
}

pub mod parallel {
    pub type Config = super::Config<super::Parallel>;
    pub type ConfigBuilder = super::ConfigBuilder<super::Parallel>;

    /// A single step in the heater profile for parallel mode.
    pub struct HeaterStep {
        /// The target temperature for the heater in degrees Celsius.
        pub(crate) target_temp: u16,
        /// The repetition multiplier for the heater step. This determines how many times this step will be repeated during the measurement cycle.
        pub(crate) multiplier: u8,
    }

    impl HeaterStep {
        /// Create a new `HeaterStep` with the specified target temperature and repetition multiplier.
        ///
        /// # Arguments
        /// * `target_temp` - The target temperature for the heater in degrees Celsius.
        /// * `multiplier` - The repetition multiplier for the heater step.
        ///
        /// # Returns
        /// A new `HeaterStep` instance.
        pub fn new(target_temp: u16, multiplier: u8) -> Self {
            Self {
                target_temp,
                multiplier,
            }
        }
    }

    impl super::ConfigBuilder<super::Parallel> {
        /// Set the heater profile for parallel mode.
        ///
        /// # Arguments
        /// * `heater_profile` - A slice of `HeaterStep` instances defining the heater profile.
        /// * `shared_heater_duration_ms` - The total duration in milliseconds for which the heater will be active during a measurement cycle.
        ///
        /// # Returns
        /// The updated `ConfigBuilder` instance.
        pub fn with_heater_profile(
            mut self,
            heater_profile: &[HeaterStep],
            shared_heater_duration_ms: u32,
        ) -> Self {
            self.config.shared_heater_duration_ms = shared_heater_duration_ms;
            self.config.heater_profile = heater_profile
                .iter()
                .map(|heater_step| super::HeaterStep {
                    target_temp: heater_step.target_temp,
                    gas_wait: heater_step.multiplier,
                })
                .collect::<heapless::Vec<super::HeaterStep, { crate::registers::MAX_HEATER_PROFILES }>>(
                );
            self
        }
    }
}

pub mod sequential {
    pub type Config = super::Config<super::Sequential>;
    pub type ConfigBuilder = super::ConfigBuilder<super::Sequential>;

    /// A single step in the heater profile for sequential mode.
    pub struct HeaterStep {
        /// The target temperature for the heater in degrees Celsius.
        pub(crate) target_temp: u16,
        /// The duration for which the heater should be active in milliseconds.
        pub(crate) duration_ms: u16,
    }

    impl HeaterStep {
        /// Create a new `HeaterStep` with the specified target temperature and duration.
        ///
        /// # Arguments
        /// * `target_temp` - The target temperature for the heater in degrees Celsius.
        /// * `duration_ms` - The duration for which the heater should be active in milliseconds.
        ///
        /// # Returns
        /// A new `HeaterStep` instance.
        pub fn new(target_temp: u16, duration_ms: u16) -> Self {
            Self {
                target_temp,
                duration_ms,
            }
        }
    }

    impl super::ConfigBuilder<super::Sequential> {
        /// Set the heater profile for sequential mode.
        ///
        /// # Arguments
        /// * `heater_profile` - A slice of `HeaterStep` instances defining the heater profile.
        /// * `standby_time` - The duration for which the sensor will be in standby between measurements.
        ///
        /// # Returns
        /// The updated `ConfigBuilder` instance.
        pub fn with_heater_profile(
            mut self,
            heater_profile: &[HeaterStep],
            standby_time: Option<crate::StandbyTime>,
        ) -> Self {
            self.config.heater_profile = heater_profile
                .iter()
                .map(|heater_step| super::HeaterStep {
                    target_temp: heater_step.target_temp,
                    gas_wait: crate::math::calc_gas_wait_time(heater_step.duration_ms),
                })
                .collect::<heapless::Vec<super::HeaterStep, { crate::registers::MAX_HEATER_PROFILES }>>(
                );
            self.config.standby_time = standby_time;
            self
        }
    }
}
