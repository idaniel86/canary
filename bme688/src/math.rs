use crate::Coeffs;

const MAX_HEATER_DURATION_MS: u16 = 4032;
const MAX_SHARED_HEATER_DURATION_MS: u16 = 1923;
const MAX_HEATER_TEMP_C: u16 = 400;
const MIN_HEATER_TEMP_C: u16 = 200;

/// Calculate the compensated temperature from the sensor.
///
/// # Arguments
/// * `temp_adc` - The raw temperature ADC value.
/// * `coeffs` - The calibration coefficients for the sensor.
///
/// # Returns
/// The compensated temperature in degrees Celsius and the fine temperature value used for pressure compensation.
pub fn calc_temperature(temp_adc: u32, coeffs: &Coeffs) -> (f32, f32) {
    let temp_adc = temp_adc as f32;

    let var1 = (temp_adc / 16384.0 - f32::from(coeffs.t1) / 1024.0) * f32::from(coeffs.t2);
    let var2 = ((temp_adc / 131072.0 - f32::from(coeffs.t1) / 8192.0)
        * (temp_adc / 131072.0 - f32::from(coeffs.t1) / 8192.0))
        * f32::from(coeffs.t3)
        * 16.0;
    let temp_fine = var1 + var2;
    let temperature = temp_fine / 5120.0;
    (temperature, temp_fine)
}

/// Calculate the compensated pressure from the sensor.
///
/// # Arguments
/// * `press_adc` - The raw pressure ADC value.
/// * `temp_fine` - The fine temperature value obtained from the temperature compensation.
/// * `coeffs` - The calibration coefficients for the sensor.
///
/// # Returns
/// The compensated pressure in Pascals.
pub fn calc_pressure(press_adc: u32, temp_fine: f32, coeffs: &Coeffs) -> f32 {
    let press_adc = press_adc as f32;

    let var1 = temp_fine / 2.0 - 64000.0;
    let var2 = var1 * var1 * (f32::from(coeffs.p6) / 131072.0) + var1 * f32::from(coeffs.p5) * 2.0;
    let var2 = var2 / 4.0 + f32::from(coeffs.p4) * 65536.0;
    let var1 =
        ((f32::from(coeffs.p3) * var1 * var1) / 16384.0 + f32::from(coeffs.p2) * var1) / 524288.0;
    let var1 = (1.0 + var1 / 32768.0) * f32::from(coeffs.p1);
    if var1 as i32 == 0 {
        return 0.0; // Avoid division by zero
    }

    let pressure = 1048576.0 - press_adc;
    let pressure = ((pressure - var2 / 4096.0) * 6250.0) / var1;
    let var1 = f32::from(coeffs.p9) * pressure * pressure / 2147483648.0;
    let var2 = pressure * f32::from(coeffs.p8) / 32768.0;
    let var3 = (pressure / 256.0) * (pressure / 256.0) * (pressure / 256.0) * f32::from(coeffs.p10)
        / 131072.0;
    let pressure = pressure + (var1 + var2 + var3 + f32::from(coeffs.p7) * 128.0) / 16.0;

    pressure
}

/// Calculate the compensated humidity from the sensor.
///
/// # Arguments
/// * `hum_adc` - The raw humidity ADC value.
/// * `temp_fine` - The fine temperature value obtained from the temperature compensation.
/// * `coeffs` - The calibration coefficients for the sensor.
///
/// # Returns
/// The compensated humidity in percentage x 1000.
pub fn calc_humidity(hum_adc: u16, temp_fine: f32, coeffs: &Coeffs) -> f32 {
    let hum_adc = hum_adc as f32;

    let temperature = temp_fine / 5120.0;
    let var1 = hum_adc - (f32::from(coeffs.h1) * 16.0 + (f32::from(coeffs.h3) / 2.0) * temperature);
    let var2 = var1
        * (f32::from(coeffs.h2) / 262144.0
            * (1.0
                + f32::from(coeffs.h4) / 16384.0 * temperature
                + f32::from(coeffs.h5) / 1048576.0 * temperature * temperature));
    let var3 = f32::from(coeffs.h6) / 16384.0;
    let var4 = f32::from(coeffs.h7) / 2097152.0;
    let humidity = var2 + (var3 + var4 * temperature) * var2 * var2;

    humidity.clamp(0.0, 100000.0)
}

/// Calculate the gas resistance from the sensor.
///
/// # Arguments
/// * `gas_adc` - The raw gas ADC value.
/// * `gas_range` - The gas range value obtained from the sensor.
///
/// # Returns
/// The calculated gas resistance in Ohms.
pub fn calc_gas_resistance(gas_adc: u16, gas_range: u8) -> f32 {
    let var1 = (262144 >> gas_range) as f32;
    let var2 = (gas_adc as f32 - 512.0) * 3.0 + 4096.0;
    let gas_resistance = 1000000. * (var1 / var2);
    gas_resistance
}

/// Calculate the heater resistance for the sensor based on the target temperature and ambient temperature.
///
/// # Arguments
/// * `target_temp` - The target temperature for the heater in degrees Celsius.
/// * `ambient_temp` - The ambient temperature in degrees Celsius.
/// * `coeffs` - The calibration coefficients for the sensor.
///
/// # Returns
/// The calculated heater resistance value to be set in the sensor.
pub fn calc_heater_resistance(target_temp: u16, ambient_temp: i16, coeffs: &Coeffs) -> u8 {
    let target_temp = target_temp.clamp(MIN_HEATER_TEMP_C, MAX_HEATER_TEMP_C);
    let var1 = ((i32::from(ambient_temp) * i32::from(coeffs.g3)) / 1000) * 256;
    let var2 = (i32::from(coeffs.g1) + 784)
        * (((((i32::from(coeffs.g2) + 154009) * i32::from(target_temp) * 5) / 100) + 3276800) / 10);
    let var3 = var1 + (var2 >> 1);
    let var4 = var3 / (i32::from(coeffs.res_heat_range) + 4);
    let var5 = (131 * i32::from(coeffs.res_heat_val)) + 65536;
    let res_heat_x100 = ((var4 / var5) - 250) * 34;
    let res_heat = (res_heat_x100 + 50) / 100;
    res_heat as u8
}

/// Calculate the gas wait time for the sensor based on the desired duration in milliseconds.
///
/// # Arguments
/// * `duration` - The desired heater duration in milliseconds (1 to 4032 ms).
///
/// # Returns
/// The calculated gas wait time value to be set in the sensor.
pub fn calc_gas_wait_time(duration_ms: u16) -> u8 {
    let mut factor = 0;
    let mut duration_ms = duration_ms.clamp(1, MAX_HEATER_DURATION_MS);

    if duration_ms < MAX_HEATER_DURATION_MS {
        while duration_ms > 0x3F {
            duration_ms >>= 2;
            factor += 1;
        }
        ((factor << 6) | duration_ms) as u8
    } else {
        0xFF
    }
}

/// Calculate the shared gas wait duration for the sensor based on the desired duration in milliseconds.
///
/// # Arguments
/// * `duration_ms` - The desired shared gas wait duration in milliseconds (1 to 1923 ms).
///
/// # Returns
/// The calculated shared gas wait duration value to be set in the sensor.
pub fn calc_shared_gas_wait_duration(duration_ms: u16) -> u8 {
    let mut factor = 0;

    if duration_ms < MAX_SHARED_HEATER_DURATION_MS {
        // Step size of 0.477ms
        let mut duration = duration_ms as u32 * 1000 / 477;
        while duration > 0x3F {
            duration >>= 2;
            factor += 1;
        }
        ((factor << 6) | duration) as u8
    } else {
        0xFF
    }
}
