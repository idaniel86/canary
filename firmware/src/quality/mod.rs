/// Represents the individual subscores for various environmental factors that contribute to the overall quality score.
#[derive(Debug, Clone, Copy, defmt::Format)]
pub struct SubScores {
    /// The subscore for CO2 concentration, which affects cognitive performance and drowsiness.
    pub co2: f32,
    /// The subscore for temperature, which affects thermal comfort and occupant complaints.
    pub temperature: f32,
    /// The subscore for humidity, which affects comfort and mold risk.
    pub humidity: f32,
    /// The subscore for illumination, which affects alertness and eye strain.
    pub light: f32,
    /// The subscore for noise, which affects concentration and comfort in various environments.
    pub noise: f32,
}

/// Represents the overall quality score, which is a weighted combination of various environmental factors.
#[derive(Debug, Clone, Copy, defmt::Format)]
pub struct QualityScore {
    /// The individual subscores for each environmental factor.
    pub subscores: SubScores,
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
    const ILLUMINATION_WEIGHT: f32 = 0.1;

    const CO2: [(f32, f32); 5] = [
        (600.0, 100.0),
        (800.0, 90.0),
        (1000.0, 60.0),
        (1500.0, 25.0),
        (2500.0, 0.0),
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

    const ILLUMINATION: [(f32, f32); 6] = [
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
            subscores: SubScores {
                co2: 0.0,
                temperature: 0.0,
                humidity: 0.0,
                light: 0.0,
                noise: 0.0,
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
        self.subscores.co2 = Self::interpolate(co2, &Self::CO2);
    }

    /// Sets the temperature subscore based on the provided temperature value.
    /// 
    /// # Arguments
    /// * `temperature` - The temperature in degrees Celsius.
    pub fn set_temperature(&mut self, temperature: f32) {
        self.subscores.temperature = Self::interpolate(temperature, &Self::TEMPERATURE);
    }

    /// Sets the humidity subscore based on the provided humidity value.
    /// 
    /// # Arguments
    /// * `humidity` - The relative humidity percentage (0-100).
    pub fn set_humidity(&mut self, humidity: f32) {
        self.subscores.humidity = Self::interpolate(humidity, &Self::HUMIDITY);
    }

    /// Sets the illumination subscore based on the provided light value.
    /// 
    /// # Arguments
    /// * `light` - The illumination in lux.
    pub fn set_illumination(&mut self, light: f32) {
        self.subscores.light = Self::interpolate(light, &Self::ILLUMINATION);
    }

    /// Sets the noise subscore based on the provided noise value.
    ///
    /// # Arguments
    /// * `noise` - The noise level in decibels (dB).
    pub fn set_noise(&mut self, noise: f32) {
        self.subscores.noise = Self::interpolate(noise, &Self::NOISE);
    }

    /// Calculates the overall quality score based on the weighted subscores.
    ///
    /// # Returns
    /// * `f32` - The overall quality score.
    pub fn calculate_score(&mut self) -> f32 {
        let total = self.subscores.co2 * Self::CO2_WEIGHT
            + self.subscores.temperature * Self::TEMPERATURE_WEIGHT
            + self.subscores.humidity * Self::HUMIDITY_WEIGHT
            + self.subscores.light * Self::ILLUMINATION_WEIGHT
            + self.subscores.noise * Self::NOISE_WEIGHT;
        total
    }
}
