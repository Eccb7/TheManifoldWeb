use thiserror::Error;

#[derive(Error, Debug)]
pub enum ProtocolError {
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("Invalid genome: {0}")]
    InvalidGenome(String),

    #[error("Invalid CID format: {0}")]
    InvalidCid(String),

    #[error("Agent not found: {0}")]
    AgentNotFound(String),

    #[error("Insufficient energy: required {required}, available {available}")]
    InsufficientEnergy { required: u64, available: u64 },

    #[error("Protocol error: {0}")]
    Protocol(String),
}

pub type Result<T> = std::result::Result<T, ProtocolError>;
