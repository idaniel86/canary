use crate::quality::{Subscores, ScoreConfig};
use crate::quality::models::QualityModel;

pub struct WeightedQualityModel {}

impl QualityModel for WeightedQualityModel {
    fn calculate_score(&self, subscores: &Subscores, config: &ScoreConfig) -> f32 {
        subscores.co2().score * config.co2.weight
            + subscores.temperature().score * config.temperature.weight
            + subscores.humidity().score * config.humidity.weight
            + subscores.illuminance().score * config.illuminance.weight
            + subscores.noise().score * config.noise.weight
    }
}
