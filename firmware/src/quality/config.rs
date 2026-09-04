/// Maximum number of score points allowed in the interpolation curve.
const MAX_SCORE_POINTS: usize = 10;

/// Represents a single point in the measurement-to-score interpolation curve.
#[derive(Debug, Clone, defmt::Format, serde::Serialize, serde::Deserialize)]
pub struct ScorePoint {
    /// The measured value corresponding to this score point.
    pub value: f32,
    /// The score corresponding to this measured value.
    pub score: f32,
}

/// Configuration for an environmental score factor, including its weight in the overall quality score and the measurement-to-score interpolation curve.
#[derive(Debug, Clone, defmt::Format, serde::Serialize, serde::Deserialize)]
pub struct ScoreFactorConfig {
    /// Low-pass filter time constant in seconds.
    pub filter_tau_seconds: f32,
    /// The weight of this environmental factor in the overall quality score calculation.
    pub weight: f32,
    /// Measurement-to-score interpolation curve.
    pub points: heapless::Vec<ScorePoint, MAX_SCORE_POINTS>,
}

/// Configuration for the overall quality score, including the individual environmental score factors.
#[derive(Debug, Clone, defmt::Format, serde::Serialize, serde::Deserialize)]
pub struct QualityScoreConfig {
    /// Configuration for the CO2 score factor.
    pub co2: ScoreFactorConfig,
    /// Configuration for the temperature score factor.
    pub temperature: ScoreFactorConfig,
    /// Configuration for the humidity score factor.
    pub humidity: ScoreFactorConfig,
    /// Configuration for the illuminance score factor.
    pub illuminance: ScoreFactorConfig,
    /// Configuration for the noise score factor.
    pub noise: ScoreFactorConfig,
}

impl Default for QualityScoreConfig {
    fn default() -> Self {
        Self {
            co2: ScoreFactorConfig {
                filter_tau_seconds: 30.0,
                // Strongest documented effect — direct dose-response on cognitive performance/drowsiness in enclosed rooms, and often the first sign of poor ventilation.
                weight: 0.3,
                points: heapless::Vec::from_array([
                    ScorePoint {
                        value: 400.0,
                        score: 100.0,
                    },
                    ScorePoint {
                        value: 600.0,
                        score: 95.0,
                    },
                    ScorePoint {
                        value: 800.0,
                        score: 90.0,
                    },
                    ScorePoint {
                        value: 1000.0,
                        score: 80.0,
                    },
                    ScorePoint {
                        value: 1400.0,
                        score: 60.0,
                    },
                    ScorePoint {
                        value: 2000.0,
                        score: 30.0,
                    },
                    ScorePoint {
                        value: 3000.0,
                        score: 10.0,
                    },
                    ScorePoint {
                        value: 5000.0,
                        score: 0.0,
                    },
                ]),
            },
            temperature: ScoreFactorConfig {
                filter_tau_seconds: 30.0,
                // Thermal comfort is consistently the top driver of occupant complaints.
                weight: 0.25,
                points: heapless::Vec::from_array([
                    ScorePoint {
                        value: 16.0,
                        score: 0.0,
                    },
                    ScorePoint {
                        value: 19.0,
                        score: 60.0,
                    },
                    ScorePoint {
                        value: 21.0,
                        score: 100.0,
                    },
                    ScorePoint {
                        value: 24.0,
                        score: 100.0,
                    },
                    ScorePoint {
                        value: 26.0,
                        score: 60.0,
                    },
                    ScorePoint {
                        value: 29.0,
                        score: 0.0,
                    },
                ]),
            },
            humidity: ScoreFactorConfig {
                filter_tau_seconds: 30.0,
                // Matters mostly at extremes (dryness/irritation, mold risk); smaller comfort range sensitivity than temp.
                weight: 0.15,
                points: heapless::Vec::from_array([
                    ScorePoint {
                        value: 15.0,
                        score: 0.0,
                    },
                    ScorePoint {
                        value: 30.0,
                        score: 60.0,
                    },
                    ScorePoint {
                        value: 45.0,
                        score: 100.0,
                    },
                    ScorePoint {
                        value: 55.0,
                        score: 100.0,
                    },
                    ScorePoint {
                        value: 70.0,
                        score: 60.0,
                    },
                    ScorePoint {
                        value: 85.0,
                        score: 0.0,
                    },
                ]),
            },
            illuminance: ScoreFactorConfig {
                filter_tau_seconds: 5.0,
                // Affects alertness/eye strain, but easiest for occupants to compensate for (blinds, desk lamps).
                weight: 0.1,
                points: heapless::Vec::from_array([
                    ScorePoint {
                        value: 0.0,
                        score: 0.0,
                    },
                    ScorePoint {
                        value: 100.0,
                        score: 20.0,
                    },
                    ScorePoint {
                        value: 200.0,
                        score: 60.0,
                    },
                    ScorePoint {
                        value: 300.0,
                        score: 90.0,
                    },
                    ScorePoint {
                        value: 500.0,
                        score: 100.0,
                    },
                    ScorePoint {
                        value: 1000.0,
                        score: 50.0,
                    },
                ]),
            },
            noise: ScoreFactorConfig {
                filter_tau_seconds: 10.0,
                // Noise affects concentration and comfort, but is often mitigated by building design and personal strategies (earplugs, headphones).
                weight: 0.2,
                points: heapless::Vec::from_array([
                    ScorePoint {
                        value: 0.0,
                        score: 100.0,
                    },
                    ScorePoint {
                        value: 40.0,
                        score: 100.0,
                    },
                    ScorePoint {
                        value: 50.0,
                        score: 80.0,
                    },
                    ScorePoint {
                        value: 60.0,
                        score: 50.0,
                    },
                    ScorePoint {
                        value: 70.0,
                        score: 20.0,
                    },
                    ScorePoint {
                        value: 80.0,
                        score: 0.0,
                    },
                ]),
            },
        }
    }
}
