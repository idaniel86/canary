/// Maximum number of score points allowed in the interpolation curve.
const MAX_SCORE_POINTS: usize = 10;

type ScoreCurve = heapless::Vec<ScorePoint, MAX_SCORE_POINTS>;

const TEMPERATURE_CURVE: ScoreCurve = heapless::Vec::from_array([
    ScorePoint {
        value: 14.0,
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
        score: 80.0,
    },
    ScorePoint {
        value: 28.0,
        score: 60.0,
    },
    ScorePoint {
        value: 30.0,
        score: 30.0,
    },
    ScorePoint {
        value: 35.0,
        score: 0.0,
    },
]);

const CO2_CURVE: ScoreCurve = heapless::Vec::from_array([
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
]);

const HUMIDITY_CURVE: ScoreCurve = heapless::Vec::from_array([
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
]);

const ILLUMINANCE_CURVE: ScoreCurve = heapless::Vec::from_array([
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
]);

const NOISE_CURVE: ScoreCurve = heapless::Vec::from_array([
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
]);

const CO2: ScoreFactorConfig = ScoreFactorConfig {
    filter_tau_seconds: 30.0,
    weight: 0.25,
    curve: CO2_CURVE,
    penalty: PenaltyConfig {
        threshold: 60.0,
        strength: 2.5,
        severity: 0.9,
    },
};

const TEMPERATURE: ScoreFactorConfig = ScoreFactorConfig {
    filter_tau_seconds: 50.0,
    weight: 0.3,
    curve: TEMPERATURE_CURVE,
    penalty: PenaltyConfig {
        threshold: 60.0,
        strength: 3.0,
        severity: 1.0,
    },
};

const HUMIDITY: ScoreFactorConfig = ScoreFactorConfig {
    filter_tau_seconds: 50.0,
    weight: 0.15,
    curve: HUMIDITY_CURVE,
    penalty: PenaltyConfig {
        threshold: 50.0,
        strength: 1.5,
        severity: 0.5,
    },
};

const ILLUMINANCE: ScoreFactorConfig = ScoreFactorConfig {
    filter_tau_seconds: 5.0,
    weight: 0.1,
    curve: ILLUMINANCE_CURVE,
    penalty: PenaltyConfig {
        threshold: 0.0,
        strength: 1.0,
        severity: 0.0,
    },
};

const NOISE: ScoreFactorConfig = ScoreFactorConfig {
    filter_tau_seconds: 10.0,
    weight: 0.2,
    curve: NOISE_CURVE,
    penalty: PenaltyConfig {
        threshold: 60.0,
        strength: 2.0,
        severity: 0.7,
    },
};

/// Represents a single point in the measurement-to-score interpolation curve.
#[derive(Debug, Clone, defmt::Format, serde::Serialize, serde::Deserialize)]
pub struct ScorePoint {
    /// The measured value corresponding to this score point.
    pub value: f32,
    /// The score corresponding to this measured value.
    pub score: f32,
}

#[derive(Debug, Clone, defmt::Format, serde::Serialize, serde::Deserialize)]
pub struct PenaltyConfig {
    /// The threshold beyond which a penalty is applied to the score.
    pub threshold: f32,
    /// Shape of the penalty curve
    pub strength: f32,
    /// How much this factor can affect the final score.
    pub severity: f32,
}

/// Configuration for an environmental score factor, including its weight in the overall quality score and the measurement-to-score interpolation curve.
#[derive(Debug, Clone, defmt::Format, serde::Serialize, serde::Deserialize)]
pub struct ScoreFactorConfig {
    /// Low-pass filter time constant in seconds.
    pub filter_tau_seconds: f32,
    /// The weight of this environmental factor in the overall quality score calculation.
    pub weight: f32,
    /// Measurement-to-score interpolation curve.
    pub curve: ScoreCurve,
    /// Penalty configuration for this environmental factor.
    pub penalty: PenaltyConfig,
}

/// Configuration for the overall quality score, including the individual environmental score factors.
#[derive(Debug, Clone, defmt::Format, serde::Serialize, serde::Deserialize)]
pub struct ScoreConfig {
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

impl Default for ScoreConfig {
    fn default() -> Self {
        Self {
            co2: CO2,
            temperature: TEMPERATURE,
            humidity: HUMIDITY,
            illuminance: ILLUMINANCE,
            noise: NOISE,
        }
    }
}
