/// Represents a sensor reading, which can be of various types such as temperature, humidity, pressure, gas resistance, CO2 levels, or light intensity (lux).
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
#[serde(tag = "type")]
pub enum Sensor {
    Temperature { value: f32 },
    Humidity { value: f32 },
    Pressure { value: u32 },
    GasResistance { profile: u32, resistance: u32 },
    CO2 { value: u32 },
    Lux { value: u32 },
    Noise { value: f32 },
}

/// Represents a reading from a sensor, including the timestamp of the reading and the type of sensor data.
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct SensorReading {
    /// The timestamp of the sensor reading in milliseconds since the Unix epoch.
    pub timestamp: u64,
    /// The type of sensor data being read, which can be temperature, humidity, pressure, gas resistance, CO2 levels, or light intensity (lux).
    pub sensor: Sensor,
}
