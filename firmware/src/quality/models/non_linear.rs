use defmt::info;

use crate::quality::models::QualityModel;
use crate::quality::{ScoreConfig, Subscores};

pub struct NonLinearQualityModel {}

impl QualityModel for NonLinearQualityModel {
    fn calculate_score(&self, subscores: &Subscores, config: &ScoreConfig) -> f32 {
        // Weighted geometric mean
        let co2_score = subscores.co2().score;
        let temperature_score = subscores.temperature().score;
        let humidity_score = subscores.humidity().score;
        let illuminance_score = subscores.illuminance().score;
        let noise_score = subscores.noise().score;

        let co2_penalty = if co2_score > config.co2.penalty.threshold {
            1.0
        } else {
            libm::powf(
                co2_score / config.co2.penalty.threshold,
                config.co2.penalty.strength,
            )
        };
        let temperature_penalty = if temperature_score > config.temperature.penalty.threshold {
            1.0
        } else {
            libm::powf(
                temperature_score / config.temperature.penalty.threshold,
                config.temperature.penalty.strength,
            )
        };

        let humidity_penalty = if humidity_score > config.humidity.penalty.threshold {
            1.0
        } else {
            libm::powf(
                humidity_score / config.humidity.penalty.threshold,
                config.humidity.penalty.strength,
            )
        };

        let illuminance_penalty = if illuminance_score > config.illuminance.penalty.threshold {
            1.0
        } else {
            libm::powf(
                illuminance_score / config.illuminance.penalty.threshold,
                config.illuminance.penalty.strength,
            )
        };

        let noise_penalty = if noise_score > config.noise.penalty.threshold {
            1.0
        } else {
            libm::powf(
                noise_score / config.noise.penalty.threshold,
                config.noise.penalty.strength,
            )
        };

        let base = libm::powf(co2_score, config.co2.weight)
            * libm::powf(temperature_score, config.temperature.weight)
            * libm::powf(humidity_score, config.humidity.weight)
            * libm::powf(illuminance_score, config.illuminance.weight)
            * libm::powf(noise_score, config.noise.weight);

        info!("Base score: {}", base);

        let penalty = libm::powf(co2_penalty, config.co2.penalty.severity)
            * libm::powf(temperature_penalty, config.temperature.penalty.severity)
            * libm::powf(humidity_penalty, config.humidity.penalty.severity)
            * libm::powf(illuminance_penalty, config.illuminance.penalty.severity)
            * libm::powf(noise_penalty, config.noise.penalty.severity);

        // Additional nonlinear penalty
        info!("Penalty: {}", penalty);
        let final_score = base * penalty;

        info!("Final score: {}", final_score);
        final_score
    }
}
