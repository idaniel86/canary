mod score;

/// Represents the default score curves for various environmental factors that contribute to the overall quality score.
const DEFAULT_SCORE_CURVES: score::ScoreCurves = score::ScoreCurves {
    co2_curve: score::ScoreCurve {
        // Strongest documented effect — direct dose-response on cognitive performance/drowsiness in enclosed rooms,
        // and often the first sign of poor ventilation.
        weight: 0.3,
        points: heapless::Vec::from_array([
            score::ScorePoint { value: 400.0, score: 100.0 },
            score::ScorePoint { value: 600.0, score: 95.0 },
            score::ScorePoint { value: 800.0, score: 90.0 },
            score::ScorePoint { value: 1000.0, score: 80.0 },
            score::ScorePoint { value: 1400.0, score: 60.0 },
            score::ScorePoint { value: 2000.0, score: 30.0 },
            score::ScorePoint { value: 3000.0, score: 10.0 },
            score::ScorePoint { value: 5000.0, score: 0.0 },
        ]),
    },
    temperature_curve: score::ScoreCurve {
        // Thermal comfort is consistently the top driver of occupant complaints.
        weight: 0.25,
        points: heapless::Vec::from_array([
            score::ScorePoint { value: 16.0, score: 0.0 },
            score::ScorePoint { value: 19.0, score: 60.0 },
            score::ScorePoint { value: 21.0, score: 100.0 },
            score::ScorePoint { value: 24.0, score: 100.0 },
            score::ScorePoint { value: 26.0, score: 60.0 },
            score::ScorePoint { value: 29.0, score: 0.0 },
        ]),
    },
    humidity_curve: score::ScoreCurve {
        // Matters mostly at extremes (dryness/irritation, mold risk); smaller comfort range sensitivity than temp.
        weight: 0.15,
        points: heapless::Vec::from_array([
            score::ScorePoint { value: 15.0, score: 0.0 },
            score::ScorePoint { value: 30.0, score: 60.0 },
            score::ScorePoint { value: 45.0, score: 100.0 },
            score::ScorePoint { value: 55.0, score: 100.0 },
            score::ScorePoint { value: 70.0, score: 60.0 },
            score::ScorePoint { value: 85.0, score: 0.0 },
        ]),
    },
    illuminance_curve: score::ScoreCurve {
        // Affects alertness/eye strain, but easiest for occupants to compensate for (blinds, desk lamps).
        weight: 0.1,
        points: heapless::Vec::from_array([
            score::ScorePoint { value: 0.0, score: 0.0 },
            score::ScorePoint { value: 100.0, score: 20.0 },
            score::ScorePoint { value: 200.0, score: 60.0 },
            score::ScorePoint { value: 300.0, score: 90.0 },
            score::ScorePoint { value: 500.0, score: 100.0 },
            score::ScorePoint { value: 1000.0, score: 50.0 },
        ]),
    },
    noise_curve: score::ScoreCurve {
        // Big impact on concentration in open-plan offices, but highly context-dependent (private office vs. open floor).
        weight: 0.2,
        points: heapless::Vec::from_array([
            score::ScorePoint { value: 0.0, score: 100.0 },
            score::ScorePoint { value: 30.0, score: 100.0 },
            score::ScorePoint { value: 40.0, score: 100.0 },
            score::ScorePoint { value: 50.0, score: 80.0 },
            score::ScorePoint { value: 60.0, score: 50.0 },
            score::ScorePoint { value: 70.0, score: 20.0 },
            score::ScorePoint { value: 80.0, score: 0.0 },
        ]),
    },
};

/// Represents the individual subscores for various environmental factors that contribute to the overall quality score.
#[derive(Debug, Clone, defmt::Format)]
pub struct SubScores {
    /// The subscore for the specific environmental factor, which is a value between 0 and 100.
    pub score: f32,
    /// The actual measurement of the environmental factor, which is used to calculate the subscore.
    pub measurement: f32,
}

/// Represents the overall quality score, which is a weighted combination of various environmental factors.
#[derive(Debug, Clone, defmt::Format)]
pub struct QualityScore {
    /// The default score curves for various environmental factors that contribute to the overall quality score.
    pub score_curves: score::ScoreCurves,
    /// The overall quality score, which is a weighted combination of the individual subscores.
    pub score: f32,
    /// The subscore for CO2 concentration, which affects cognitive performance and drowsiness.
    pub co2: SubScores,
    /// The subscore for temperature, which affects thermal comfort and occupant complaints.
    pub temperature: SubScores,
    /// The subscore for humidity, which affects comfort and mold risk.
    pub humidity: SubScores,
    /// The subscore for illuminance, which affects alertness and eye strain.
    pub illuminance: SubScores,
    /// The subscore for noise, which affects concentration and comfort in various environments.
    pub noise: SubScores,
}

impl QualityScore {
    /// Creates a new `QualityScore` instance with all subscores initialized to zero.
    pub const fn new() -> Self {
        Self {
            score_curves: DEFAULT_SCORE_CURVES,
            score: 0.0,
            co2: SubScores {
                score: 0.0,
                measurement: 0.0,
            },
            temperature: SubScores {
                score: 0.0,
                measurement: 0.0,
            },
            humidity: SubScores {
                score: 0.0,
                measurement: 0.0,
            },
            illuminance: SubScores {
                score: 0.0,
                measurement: 0.0,
            },
            noise: SubScores {
                score: 0.0,
                measurement: 0.0,
            },
        }
    }

    /// Interpolates the score based on the provided value and the defined points.
    fn interpolate(value: f32, points: &[score::ScorePoint]) -> f32 {
        if let Some(&score::ScorePoint { value: x0, score: y0 }) = points.first() {
            if value <= x0 {
                return y0;
            }
        }
        if let Some(&score::ScorePoint { value: xn, score: yn }) = points.last() {
            if value >= xn {
                return yn;
            }
        }
        for window in points.windows(2) {
            let score::ScorePoint { value: x0, score: y0 } = window[0];
            let score::ScorePoint { value: x1, score: y1 } = window[1];
            if value >= x0 && value <= x1 {
                let t = (value - x0) / (x1 - x0);
                return y0 + t * (y1 - y0);
            }
        }
        0.0
    }

    /// Sets the CO2 subscore based on the provided CO2 value.
    ///
    /// # Arguments
    /// * `co2` - The CO2 concentration in parts per million (ppm).
    pub fn set_co2(&mut self, co2: f32) {
        self.co2.score = Self::interpolate(co2, self.score_curves.co2_curve.points.as_slice());
        self.co2.measurement = co2;
    }

    /// Sets the temperature subscore based on the provided temperature value.
    ///
    /// # Arguments
    /// * `temperature` - The temperature in degrees Celsius.
    pub fn set_temperature(&mut self, temperature: f32) {
        self.temperature.score = Self::interpolate(temperature, self.score_curves.temperature_curve.points.as_slice());
        self.temperature.measurement = temperature;
    }

    /// Sets the humidity subscore based on the provided humidity value.
    ///
    /// # Arguments
    /// * `humidity` - The relative humidity percentage (0-100).
    pub fn set_humidity(&mut self, humidity: f32) {
        self.humidity.score = Self::interpolate(humidity, self.score_curves.humidity_curve.points.as_slice());
        self.humidity.measurement = humidity;
    }

    /// Sets the illumination subscore based on the provided light value.
    ///
    /// # Arguments
    /// * `light` - The illumination in lux.
    pub fn set_illuminance(&mut self, illuminance: f32) {
        self.illuminance.score = Self::interpolate(illuminance, self.score_curves.illuminance_curve.points.as_slice());
        self.illuminance.measurement = illuminance;
    }

    /// Sets the noise subscore based on the provided noise value.
    ///
    /// # Arguments
    /// * `noise` - The noise level in decibels (dB).
    pub fn set_noise(&mut self, noise: f32) {
        self.noise.score = Self::interpolate(noise, self.score_curves.noise_curve.points.as_slice());
        self.noise.measurement = noise;
    }

    /// Calculates the overall quality score based on the weighted subscores.
    pub fn calculate_score(&mut self) {
        self.score = self.co2.score * self.score_curves.co2_curve.weight
            + self.temperature.score * self.score_curves.temperature_curve.weight
            + self.humidity.score * self.score_curves.humidity_curve.weight
            + self.illuminance.score * self.score_curves.illuminance_curve.weight
            + self.noise.score * self.score_curves.noise_curve.weight;
    }
}
