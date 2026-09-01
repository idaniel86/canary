/// Represents the individual subscores for various environmental factors that contribute to the overall quality score.
#[derive(Debug, Clone, Copy, serde::Deserialize, serde::Serialize)]
pub struct SubScores {
    /// The subscore for the specific environmental factor, which is a value between 0 and 100.
    pub score: f32,
    /// The actual measurement of the environmental factor, which is used to calculate the subscore.
    pub measurement: f32,
}

/// Represents the overall quality score, which is a weighted combination of various environmental factors.
#[derive(Debug, Clone, Copy, serde::Deserialize, serde::Serialize)]
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
