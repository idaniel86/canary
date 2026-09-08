#[derive(Debug, Clone, Copy, defmt::Format)]
pub enum Reading {
    Temperature(f32),
    Humidity(f32),
    Co2(f32),
    Illuminance(f32),
    Noise(f32),
}
