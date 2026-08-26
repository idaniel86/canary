/// Represents a sensor reading, which can be of various types such as temperature, humidity, pressure, gas resistance, CO2 levels, or light intensity (lux).
#[derive(Debug, Clone)]
pub enum Sensor {
    Temperature(f32),
    Humidity(f32),
    Pressure(u32),
    GasResistance { profile: u32, resistance: u32 },
    CO2(u32),
    Lux(u32),
}

/// Represents a reading from a sensor, including the timestamp of the reading and the type of sensor data.
#[derive(Debug, Clone)]
pub struct SensorReading {
    /// The timestamp of the sensor reading in milliseconds since the Unix epoch.
    pub timestamp: u64,
    /// The type of sensor data being read, which can be temperature, humidity, pressure, gas resistance, CO2 levels, or light intensity (lux).
    pub sensor: Sensor,
}
