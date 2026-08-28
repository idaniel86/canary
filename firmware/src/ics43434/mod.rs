mod filters;
use filters::Filter;

fn signed_24(raw: u32) -> i32 {
    let value = raw & 0x00ff_ffff;

    if value & 0x0080_0000 != 0 {
        (value | 0xff00_0000) as i32
    } else {
        value as i32
    }
}

pub struct Ics43434 {
    a_weighting: filters::AWeighting,
    time_weighting: filters::TimeWeighting,
    rms: f32,
}

impl Ics43434 {
    const ALPHA_FAST: f32 = 0.99983335;
    #[allow(dead_code)]
    const ALPHA_SLOW: f32 = 0.9999792;
    const REFERENCE_RMS: f32 = 0.035438; // Reference RMS value for 94 dB SPL at 1 kHz, based on the ICS43434 datasheet.
    const REFERENCE_SPL: f32 = 94.0; // Reference SPL value in dB for the ICS43434 microphone.

    /// Creates a new instance of the Ics43434 microphone interface with A-weighting and time-weighting filters.
    /// 
    /// # Returns
    /// * `Self` - A new instance of the Ics43434 microphone
    pub fn new() -> Self {
        Self {
            a_weighting: filters::AWeighting::new(),
            time_weighting: filters::TimeWeighting::new(Self::ALPHA_FAST),
            rms: 0.0,
        }
    }

    /// Processes a raw 24-bit audio sample from the ICS43434 microphone.
    ///
    /// # Arguments
    /// * `raw_sample` - The raw 24-bit audio sample as a 32-bit unsigned integer, where the audio data is in the upper 24 bits.
    pub fn process(&mut self, raw_sample: u32) {
        // The 24-bit audio is situated in the upper bits of the 32-bit container
        let sample = signed_24(raw_sample);
        // Normalize to [-1.0, 1.0]
        let normalized_sample = sample as f32 / 8388608.0;
        // Apply A-weighting and time-weighting filters
        let a_weighted_sample = self.a_weighting.process(normalized_sample);
        // Apply time-weighting filter
        let time_weighted_sample = self.time_weighting.process(a_weighted_sample);
        self.rms = time_weighted_sample;
    }

    /// Resets the internal state of the Ics43434 microphone interface, including the filters and RMS value.
    #[allow(dead_code)]
    pub fn reset(&mut self) {
        self.a_weighting.reset();
        self.time_weighting.reset();
        self.rms = 0.0;
    }

    /// Returns the current Sound Pressure Level (SPL) in decibels (dB).
    /// 
    /// # Returns
    /// * `f32` - The calculated SPL value in dB.
    pub fn get_spl(&self) -> f32 {
        Self::REFERENCE_SPL + 20.0 * libm::log10f(self.rms / Self::REFERENCE_RMS)
    }
}