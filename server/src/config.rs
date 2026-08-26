#[derive(serde::Deserialize, Debug)]
pub struct Config {
    /// The server configuration, including HTTP and TCP addresses.
    pub server: ServerConfig,
}

#[derive(serde::Deserialize, Debug)]
pub struct ServerConfig {
    /// The address on which the TCP server will listen for incoming sensor readings.
    pub sensor_address: std::net::SocketAddr,
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