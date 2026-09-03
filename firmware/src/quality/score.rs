const MAX_SCORE_CURVE_POINTS: usize = 10;

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, defmt::Format)]
pub struct ScorePoint {
    /// The measurement value for the environmental factor, which is used to determine the corresponding score.
    pub value: f32,
    /// The score value for the environmental factor, which is a value between 0 and 100.
    pub score: f32,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, defmt::Format)]
pub struct ScoreCurve {
    /// The weight of the environmental factor in the overall quality score calculation.
    pub weight: f32,
    /// The points that define the score curve for the environmental factor, where each point is a measurement-score pair.
    pub points: heapless::Vec<ScorePoint, MAX_SCORE_CURVE_POINTS>,
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize, defmt::Format)]
pub struct ScoreCurves {
    /// The score curve for CO2 concentration, which affects cognitive performance and drowsiness.
    pub co2_curve: ScoreCurve,
    /// The score curve for temperature, which affects thermal comfort and occupant complaints.
    pub temperature_curve: ScoreCurve,
    /// The score curve for humidity, which affects comfort and mold risk.
    pub humidity_curve: ScoreCurve,
    /// The score curve for illuminance, which affects alertness and eye strain.
    pub illuminance_curve: ScoreCurve,
    /// The score curve for noise, which affects concentration and comfort in various environments.
    pub noise_curve: ScoreCurve,
}