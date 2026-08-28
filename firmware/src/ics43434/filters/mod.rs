mod a_weighting;
mod time_weighting;

pub use a_weighting::AWeighting;
pub use time_weighting::TimeWeighting;

pub trait Filter {
    /// Processes a single sample through the filter.
    /// 
    /// # Arguments
    /// * `sample` - The input sample to be processed.
    /// 
    /// # Returns
    /// * `f32` - The filtered output sample after applying the filter.
    fn process(&mut self, sample: f32) -> f32;
    
    /// Resets the internal state of the filter, clearing any stored history.
    fn reset(&mut self);
}