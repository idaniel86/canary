#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Configuration error: {0}")]
    Config(#[from] config::ConfigError),

    #[error("Pipeline processing error: {0}")]
    ProcessingError(String),

    #[error("Protocol error: {0}")]
    ProtocolError(#[from] std::io::Error),
}
