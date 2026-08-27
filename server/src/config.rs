#[derive(serde::Deserialize, Debug)]
pub struct Config {
    /// The server configuration, including HTTP and TCP addresses.
    pub sensor: SensorConfig,
    /// The web server configuration, including the HTTP address.
    pub web: WebConfig,
}

#[derive(serde::Deserialize, Debug)]
pub struct SensorConfig {
    /// The address on which the TCP server will listen for incoming sensor readings.
    pub address: std::net::SocketAddr,
    pub capacity: usize,
}

#[derive(serde::Deserialize, Debug)]
pub struct WebConfig {
    /// The address on which the HTTP server will listen for incoming requests.
    pub address: std::net::SocketAddr,
}

impl Config {
    pub fn load() -> Result<Self, config::ConfigError> {
        let config = config::Config::builder()
            // Add the default configuration file (config/default) to the configuration builder.
            .add_source(config::File::with_name("config/default"))
            .build()?;

        config.try_deserialize()
    }
}
