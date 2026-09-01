/// Represents the individual subscores for various environmental factors that contribute to the overall quality score.
#[derive(Debug, Clone, Copy, defmt::Format)]
pub struct SubScores {
    /// The subscore for the specific environmental factor, which is a value between 0 and 100.
    pub score: f32,
    /// The actual measurement of the environmental factor, which is used to calculate the subscore.
    pub measurement: f32,
}

/// Represents the overall quality score, which is a weighted combination of various environmental factors.
#[derive(Debug, Clone, Copy, defmt::Format)]
pub struct QualityScore {
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
    /// Strongest documented effect — direct dose-response on cognitive performance/drowsiness in enclosed rooms,
    /// and often the first sign of poor ventilation.
    const CO2_WEIGHT: f32 = 0.3;

    /// Thermal comfort is consistently the top driver of occupant complaints.
    const TEMPERATURE_WEIGHT: f32 = 0.25;

    /// Big impact on concentration in open-plan offices, but highly context-dependent (private office vs. open floor).
    const NOISE_WEIGHT: f32 = 0.2;

    /// Matters mostly at extremes (dryness/irritation, mold risk); smaller comfort range sensitivity than temp.
    const HUMIDITY_WEIGHT: f32 = 0.15;

    /// Affects alertness/eye strain, but easiest for occupants to compensate for (blinds, desk lamps).
    const ILLUMINANCE_WEIGHT: f32 = 0.1;

    const CO2: [(f32, f32); 8] = [
        (400.0, 100.0),
        (600.0, 95.0),
        (800.0, 90.0),
        (1000.0, 80.0),
        (1400.0, 60.0),
        (2000.0, 30.0),
        (3000.0, 10.0),
        (5000.0, 0.0),
    ];

    const TEMPERATURE: [(f32, f32); 6] = [
        (16.0, 0.0),
        (19.0, 60.0),
        (21.0, 100.0),
        (24.0, 100.0),
        (26.0, 60.0),
        (29.0, 0.0),
    ];

    const HUMIDITY: [(f32, f32); 6] = [
        (15.0, 0.0),
        (30.0, 60.0),
        (45.0, 100.0),
        (55.0, 100.0),
        (70.0, 60.0),
        (85.0, 0.0),
    ];

    const ILLUMINANCE: [(f32, f32); 6] = [
        (0.0, 0.0),
        (100.0, 20.0),
        (200.0, 60.0),
        (300.0, 90.0),
        (500.0, 100.0),
        (1000.0, 50.0),
    ];

    const NOISE: [(f32, f32); 7] = [
        (0.0, 100.0),
        (30.0, 100.0),
        (40.0, 100.0),
        (50.0, 80.0),
        (60.0, 50.0),
        (70.0, 20.0),
        (80.0, 0.0),
    ];

    /// Creates a new `QualityScore` instance with all subscores initialized to zero.
    pub const fn new() -> Self {
        Self {
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
    fn interpolate(value: f32, points: &[(f32, f32)]) -> f32 {
        if let Some(&(x0, y0)) = points.first() {
            if value <= x0 {
                return y0;
            }
        }
        if let Some(&(xn, yn)) = points.last() {
            if value >= xn {
                return yn;
            }
        }
        for window in points.windows(2) {
            let (x0, y0) = window[0];
            let (x1, y1) = window[1];
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
        self.co2.score = Self::interpolate(co2, &Self::CO2);
        self.co2.measurement = co2;
    }

    /// Sets the temperature subscore based on the provided temperature value.
    ///
    /// # Arguments
    /// * `temperature` - The temperature in degrees Celsius.
    pub fn set_temperature(&mut self, temperature: f32) {
        self.temperature.score = Self::interpolate(temperature, &Self::TEMPERATURE);
        self.temperature.measurement = temperature;
    }

    /// Sets the humidity subscore based on the provided humidity value.
    ///
    /// # Arguments
    /// * `humidity` - The relative humidity percentage (0-100).
    pub fn set_humidity(&mut self, humidity: f32) {
        self.humidity.score = Self::interpolate(humidity, &Self::HUMIDITY);
        self.humidity.measurement = humidity;
    }

    /// Sets the illumination subscore based on the provided light value.
    ///
    /// # Arguments
    /// * `light` - The illumination in lux.
    pub fn set_illuminance(&mut self, illuminance: f32) {
        self.illuminance.score = Self::interpolate(illuminance, &Self::ILLUMINANCE);
        self.illuminance.measurement = illuminance;
    }

    /// Sets the noise subscore based on the provided noise value.
    ///
    /// # Arguments
    /// * `noise` - The noise level in decibels (dB).
    pub fn set_noise(&mut self, noise: f32) {
        self.noise.score = Self::interpolate(noise, &Self::NOISE);
        self.noise.measurement = noise;
    }

    /// Calculates the overall quality score based on the weighted subscores.
    pub fn calculate_score(&mut self) {
        self.score = self.co2.score * Self::CO2_WEIGHT
            + self.temperature.score * Self::TEMPERATURE_WEIGHT
            + self.humidity.score * Self::HUMIDITY_WEIGHT
            + self.illuminance.score * Self::ILLUMINANCE_WEIGHT
            + self.noise.score * Self::NOISE_WEIGHT;
    }
}
