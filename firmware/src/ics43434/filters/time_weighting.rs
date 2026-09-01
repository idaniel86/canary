use super::Filter;

/// Represents a time-weighting filter that applies an exponential moving average to audio samples.
pub struct TimeWeighting {
    alpha: f32,
    energy: f32,
}

impl TimeWeighting {
    /// Creates a new instance of the TimeWeighting filter with the specified alpha value.
    /// # Arguments
    /// * `alpha` - The smoothing factor for the exponential moving average. Should be between 0 and 1, where values closer to 1 give more weight to recent samples.
    ///
    /// # Returns
    /// * `Self` - A new instance of the TimeWeighting filter.
    pub const fn new(alpha: f32) -> Self {
        Self { alpha, energy: 0.0 }
    }
}

impl Filter for TimeWeighting {
    #[inline]
    fn process(&mut self, sample: f32) -> f32 {
        let x2 = sample * sample;
        self.energy = self.alpha * self.energy + (1.0 - self.alpha) * x2;
        libm::sqrtf(self.energy)
    }

    fn reset(&mut self) {
        self.energy = 0.0;
    }
}
