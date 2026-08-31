use biquad::{Biquad, Coefficients, DirectForm2Transposed};

use super::Filter;

const COEFFS_1: Coefficients<f32> = Coefficients {
    b0: 0.2557411252,
    b1: -0.5114822504,
    b2: 0.2557411252,
    a1: -0.1712489380,
    a2: 0.0244547327,
};

const COEFFS_2: Coefficients<f32> = Coefficients {
    b0: 1.0,
    b1: -2.0,
    b2: 1.0,
    a1: -1.9675228411,
    a2: 0.9681097730,
};

const COEFFS_3: Coefficients<f32> = Coefficients {
    b0: 1.0,
    b1: -2.0,
    b2: 1.0,
    a1: -1.9900632793,
    a2: 0.9901129139,
};

const COEFFS_4: Coefficients<f32> = Coefficients {
    b0: 1.0,
    b1: -2.0,
    b2: 1.0,
    a1: -1.9938533933,
    a2: 0.9938675313,
};

/// Represents an A-weighting filter that applies a series of biquad filters to audio samples.
///
/// The samples are sampled at 48 kHz, and the filter is designed to approximate the A-weighting curve used in sound level measurements.
pub struct AWeighting {
    filter1: DirectForm2Transposed<f32>,
    filter2: DirectForm2Transposed<f32>,
    filter3: DirectForm2Transposed<f32>,
    filter4: DirectForm2Transposed<f32>,
}

impl AWeighting {
    /// Creates a new instance of the AWeighting filter with the predefined coefficients.
    ///
    /// # Returns
    /// * `Self` - A new instance of the AWeighting filter.
    pub const fn new() -> Self {
        Self {
            filter1: DirectForm2Transposed::new(COEFFS_1),
            filter2: DirectForm2Transposed::new(COEFFS_2),
            filter3: DirectForm2Transposed::new(COEFFS_3),
            filter4: DirectForm2Transposed::new(COEFFS_4),
        }
    }
}

impl Filter for AWeighting {
    #[inline]
    fn process(&mut self, sample: f32) -> f32 {
        let sample = self.filter1.run(sample);
        let sample = self.filter2.run(sample);
        let sample = self.filter3.run(sample);
        let sample = self.filter4.run(sample);
        sample
    }

    fn reset(&mut self) {
        self.filter1.reset_state();
        self.filter2.reset_state();
        self.filter3.reset_state();
        self.filter4.reset_state();
    }
}
