/// Converts the raw result from the OPT3001 sensor to a lux value.
///
/// # Arguments
/// * `exponent` - The exponent part of the raw result from the sensor.
/// * `fractional_result` - The fractional part of the raw result from the sensor.
/// # Returns
/// The calculated lux value as a floating-point number.
pub fn to_lux(exponent: u8, fractional_result: u16) -> f32 {
    let lux = 0.01 * (1u32 << exponent) as f32 * fractional_result as f32;
    lux
}

/// Converts a lux value to the corresponding exponent and fractional result for the OPT3001 sensor.
///
/// # Arguments
/// * `lux` - The lux value to convert.
/// # Returns
/// A tuple containing the exponent and fractional result corresponding to the given lux value.
pub fn from_lux(lux: f32) -> (u8, u16) {
    let mut exp = 0u8;
    let mut fractional_result = (lux / 0.01) as u32;
    while fractional_result > 0x0FFF && exp < 11 {
        fractional_result >>= 1;
        exp += 1;
    }
    // Saturate instead of truncating when lux exceeds the max representable value.
    (exp, fractional_result.min(0x0FFF) as u16)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_lux() {
        let lux = to_lux(0, 0x0001);
        assert!((lux - 0.01).abs() <= 1e-5);

        let lux = to_lux(0, 0x0FFF);
        assert!((lux - 40.95).abs() <= 1e-5);

        let lux = to_lux(3, 0x0456);
        assert!((lux - 88.80).abs() <= 1e-5);
    }

    #[test]
    fn test_from_lux() {
        let (exp, frac) = from_lux(0.01);
        assert_eq!(exp, 0);
        assert_eq!(frac, 0x0001);

        let (exp, frac) = from_lux(40.95);
        assert_eq!(exp, 0);
        assert_eq!(frac, 0x0FFF);

        // the first match where fractional_result is less than or equal to 0x0FFF
        let (exp, frac) = from_lux(88.80);
        assert_eq!(exp, 2);
        assert_eq!(frac, 0x8AC);
    }
}
