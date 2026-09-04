use crate::quality::{QualityScoreConfig, ScorePoint};

#[derive(Debug, Clone, Default, defmt::Format, serde::Deserialize, serde::Serialize)]
pub struct Subscore {
    pub score: f32,
    pub value: f32,
}

/// Represents the overall quality score, which is a weighted combination of various environmental factors.
#[derive(Debug, Clone, Default, defmt::Format, serde::Deserialize, serde::Serialize)]
pub struct QualityScore {
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

impl QualityScore {
    /// Creates a new `QualityScore` instance with default values.
    pub fn new() -> Self {
        Default::default()
    }

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

    /// Update the CO2 subscore based on the given value and configuration.
    ///
    /// # Arguments
    /// * `value` - The CO2 value for which to update the subscore.
    /// * `config` - The configuration containing the score points for CO2 as part of the overall quality score configuration.
    pub fn update_co2(&mut self, value: f32, config: &QualityScoreConfig) {
        self.co2 = Subscore {
            value,
            score: Self::interpolate(value, &config.co2.points),
        };
        self.score = self.calc_score(config);
    }

    /// Update the temperature subscore based on the given value and configuration.
    ///
    /// # Arguments
    /// * `value` - The temperature value for which to update the subscore.
    /// * `config` - The configuration containing the score points for temperature as part of the overall quality score configuration.
    pub fn update_temperature(&mut self, value: f32, config: &QualityScoreConfig) {
        self.temperature = Subscore {
            value,
            score: Self::interpolate(value, &config.temperature.points),
        };
        self.score = self.calc_score(config);
    }

    /// Update the humidity subscore based on the given value and configuration.
    ///
    /// # Arguments
    /// * `value` - The humidity value for which to update the subscore.
    /// * `config` - The configuration containing the score points for humidity as part of the overall quality score configuration.
    pub fn update_humidity(&mut self, value: f32, config: &QualityScoreConfig) {
        self.humidity = Subscore {
            value,
            score: Self::interpolate(value, &config.humidity.points),
        };
        self.score = self.calc_score(config);
    }

    /// Update the illuminance subscore based on the given value and configuration.
    ///
    /// # Arguments
    /// * `value` - The illuminance value for which to update the subscore.
    /// * `config` - The configuration containing the score points for illuminance as part of the overall quality score configuration.
    pub fn update_illuminance(&mut self, value: f32, config: &QualityScoreConfig) {
        self.illuminance = Subscore {
            value,
            score: Self::interpolate(value, &config.illuminance.points),
        };
        self.score = self.calc_score(config);
    }

    /// Update the noise subscore based on the given value and configuration.
    ///
    /// # Arguments
    /// * `value` - The noise value for which to update the subscore.
    /// * `config` - The configuration containing the score points for noise as part of the overall quality score configuration.
    pub fn update_noise(&mut self, value: f32, config: &QualityScoreConfig) {
        self.noise = Subscore {
            value,
            score: Self::interpolate(value, &config.noise.points),
        };
        self.score = self.calc_score(config);
    }

    /// Calculate the overall quality score based on the individual subscores and their respective weights.
    ///
    /// # Arguments
    /// * `config` - The configuration containing the weights for each subscore.
    ///
    /// # Returns
    /// The overall quality score as a floating-point value.
    fn calc_score(&self, config: &QualityScoreConfig) -> f32 {
        self.co2.score * config.co2.weight
            + self.temperature.score * config.temperature.weight
            + self.humidity.score * config.humidity.weight
            + self.illuminance.score * config.illuminance.weight
            + self.noise.score * config.noise.weight
    }
}
