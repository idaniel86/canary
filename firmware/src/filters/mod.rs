use embassy_time::Instant;

/// A simple low-pass filter implementation that smooths out input values over time.
///
/// Using a single-pole IIR low-pass (a.k.a. exponential moving average / RC low-pass equivalent),
/// with the smoothing factor derived from elapsed time rather than fixed per sample.
pub struct LowPassFilter {
    tau: f32,
    last_time: Instant,
    prev_output: Option<f32>,
}

impl LowPassFilter {
    /// Creates a new instance of the LowPassFilter with a specified time constant and initial output value.
    ///
    /// # Arguments
    /// * `tau` - The time constant for the low-pass filter, which determines how quickly the filter responds to changes in input.
    /// * `initial_output` - The initial output value of the filter, which is used as the starting point for filtering.
    ///
    /// # Returns
    /// * `Self` - A new instance of the LowPassFilter.
    pub fn new(tau: f32, initial_output: Option<f32>) -> Self {
        Self {
            tau,
            last_time: Instant::now(),
            prev_output: initial_output,
        }
    }

    /// Processes a new input value through the low-pass filter and returns the filtered output.
    ///
    /// # Arguments
    /// * `input` - The new input value to be filtered.
    ///
    /// # Returns
    /// * `f32` - The filtered output value after applying the low-pass filter to the input.
    pub fn process(&mut self, input: f32) -> f32 {
        let now = Instant::now();
        let dt = (now - self.last_time).as_millis() as f32 / 1000.0; // Convert milliseconds to seconds
        self.last_time = now;
        let alpha = 1.0 - libm::expf(-dt / self.tau);
        self.prev_output = Some(match self.prev_output {
            Some(prev) => prev + alpha * (input - prev),
            None => input,
        });
        self.prev_output.unwrap()
    }
}
