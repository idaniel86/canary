use crate::{
    filters::LowPassFilter,
    quality::{Reading, ScoreConfig, ScorePoint, models::QualityModel},
};

/// Interpolates the score for a given value based on the provided score points.
///
/// # Arguments
/// * `value` - The value for which to interpolate the score.
/// * `points` - A slice of `ScorePoint` structs representing the mapping of values to scores.
///
/// # Returns
/// The interpolated score as a `f32`. If the value is outside the range of the provided points,
/// the score corresponding to the nearest endpoint is returned.
fn interpolate(value: f32, points: &[ScorePoint]) -> f32 {
    if let Some(&ScorePoint {
        value: x0,
        score: y0,
    }) = points.first()
    {
        if value <= x0 {
            return y0;
        }
    }
    if let Some(&ScorePoint {
        value: xn,
        score: yn,
    }) = points.last()
    {
        if value >= xn {
            return yn;
        }
    }
    for window in points.windows(2) {
        let ScorePoint {
            value: x0,
            score: y0,
        } = window[0];
        let ScorePoint {
            value: x1,
            score: y1,
        } = window[1];
        if value >= x0 && value <= x1 {
            let t = (value - x0) / (x1 - x0);
            return y0 + t * (y1 - y0);
        }
    }
    0.0
}

/// Represents the subscore for an individual environmental factor.
#[derive(Debug, Clone, Default, defmt::Format, serde::Deserialize, serde::Serialize)]
pub struct Subscore {
    /// The subscore for this particular environmental factor.
    pub score: f32,
    /// The actual measured value for this environmental factor.
    pub value: f32,
}
/// Maintains the state for an individual environmental factor, including its low-pass filter and subscore.
struct FactorState {
    /// The low-pass filter used to smooth the incoming sensor readings.
    filter: LowPassFilter,
    /// The subscore for this environmental factor.
    subscore: Subscore,
}

impl FactorState {
    /// Creates a new `FactorState`
    ///
    /// # Arguments
    /// * `filter_tau_seconds` - The time constant for the low-pass filter applied to this environmental factor.
    ///
    /// # Returns
    /// A new instance of `FactorState`..
    pub fn new(filter_tau_seconds: f32) -> Self {
        Self {
            filter: LowPassFilter::new(filter_tau_seconds),
            subscore: Subscore::default(),
        }
    }

    /// Updates the `FactorState` with a new sensor reading and recalculates the subscore based on the provided score points.
    ///
    /// # Arguments
    /// * `value` - The new sensor reading for this environmental factor.
    /// * `points` - The score points used to interpolate the subscore.
    pub fn update(&mut self, value: f32, points: &[ScorePoint]) {
        let filtered_value = self.filter.process(value);
        self.subscore.value = filtered_value;
        self.subscore.score = interpolate(filtered_value, points);
    }
}

/// Maintains the state for all environmental factors, providing access to their subscores and update functionality.
pub struct Subscores {
    /// The state for the CO2 environmental factor.
    co2: FactorState,
    /// The state for the temperature environmental factor.
    temperature: FactorState,
    /// The state for the humidity environmental factor.
    humidity: FactorState,
    /// The state for the illuminance environmental factor.
    illuminance: FactorState,
    /// The state for the noise environmental factor.
    noise: FactorState,
}

impl Subscores {
    pub fn new(config: &ScoreConfig) -> Self {
        Self {
            co2: FactorState::new(config.co2.filter_tau_seconds),
            temperature: FactorState::new(config.temperature.filter_tau_seconds),
            humidity: FactorState::new(config.humidity.filter_tau_seconds),
            illuminance: FactorState::new(config.illuminance.filter_tau_seconds),
            noise: FactorState::new(config.noise.filter_tau_seconds),
        }
    }

    pub fn co2(&self) -> &Subscore {
        &self.co2.subscore
    }

    pub fn temperature(&self) -> &Subscore {
        &self.temperature.subscore
    }

    pub fn humidity(&self) -> &Subscore {
        &self.humidity.subscore
    }

    pub fn illuminance(&self) -> &Subscore {
        &self.illuminance.subscore
    }

    pub fn noise(&self) -> &Subscore {
        &self.noise.subscore
    }

    pub fn update(&mut self, reading: Reading, config: &ScoreConfig) {
        match reading {
            Reading::Co2(value) => {
                self.co2.update(value, &config.co2.curve);
            }
            Reading::Temperature(value) => {
                self.temperature.update(value, &config.temperature.curve);
            }
            Reading::Humidity(value) => {
                self.humidity.update(value, &config.humidity.curve);
            }
            Reading::Illuminance(value) => {
                self.illuminance.update(value, &config.illuminance.curve);
            }
            Reading::Noise(value) => {
                self.noise.update(value, &config.noise.curve);
            }
        }
    }

    pub fn snapshot(&self, model: &impl QualityModel, config: &ScoreConfig) -> Score {
        Score {
            score: model.calculate_score(self, config),
            co2: self.co2.subscore.clone(),
            temperature: self.temperature.subscore.clone(),
            humidity: self.humidity.subscore.clone(),
            illuminance: self.illuminance.subscore.clone(),
            noise: self.noise.subscore.clone(),
        }
    }
}

/// Represents the overall quality score, which is a weighted combination of various environmental factors.
#[derive(Debug, Clone, Default, defmt::Format, serde::Deserialize, serde::Serialize)]
pub struct Score {
    /// The overall quality score, which is a weighted combination of various environmental factors.
    pub score: f32,
    /// The subscore for CO2 concentration, which affects cognitive performance and drowsiness.
    pub co2: Subscore,
    /// The subscore for temperature, which affects thermal comfort and occupant complaints.
    pub temperature: Subscore,
    /// The subscore for humidity, which affects comfort and mold risk.
    pub humidity: Subscore,
    /// The subscore for illuminance, which affects alertness and eye strain.
    pub illuminance: Subscore,
    /// The subscore for noise, which affects concentration and comfort in various environments.
    pub noise: Subscore,
}
