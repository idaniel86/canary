use crate::quality::{ScoreConfig, Subscores};

pub mod non_linear;
pub mod weighted;

/// A trait representing a quality model for evaluating environmental conditions.
pub trait QualityModel {
    /// Calculates the quality score of the environment based on the model.
    ///
    /// # Arguments
    /// * `environment` - A reference to the current environmental conditions.
    ///
    /// # Returns
    /// The calculated quality score as a `f32` value.
    fn calculate_score(&self, subscores: &Subscores, config: &ScoreConfig) -> f32;
}

pub enum AnyQualityModel {
    Weighted(weighted::WeightedQualityModel),
    NonLinear(non_linear::NonLinearQualityModel),
}

impl QualityModel for AnyQualityModel {
    fn calculate_score(&self, subscores: &Subscores, config: &ScoreConfig) -> f32 {
        match self {
            AnyQualityModel::Weighted(model) => model.calculate_score(subscores, config),
            AnyQualityModel::NonLinear(model) => model.calculate_score(subscores, config),
        }
    }
}
