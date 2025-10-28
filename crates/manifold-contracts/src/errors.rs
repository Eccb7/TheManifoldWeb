//! Contract error types

use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum ContractError {
    #[error("Contract not found: {0}")]
    NotFound(String),

    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error("Insufficient gas: required {required}, available {available}")]
    OutOfGas { required: u64, available: u64 },

    #[error("Invalid state: {0}")]
    InvalidState(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Storage error: {0}")]
    StorageError(String),

    #[error("Execution failed: {0}")]
    ExecutionFailed(String),

    #[error("Invalid address: {0}")]
    InvalidAddress(String),

    #[error("Contract already exists: {0}")]
    AlreadyExists(String),

    #[error("Migration failed: {0}")]
    MigrationFailed(String),

    #[error("Custom error: {0}")]
    Custom(String),
    
    #[error("Insufficient funds: {0}")]
    InsufficientFunds(String),
    
    #[error("Invalid input: {0}")]
    InvalidInput(String),
}

impl From<serde_json::Error> for ContractError {
    fn from(err: serde_json::Error) -> Self {
        ContractError::SerializationError(err.to_string())
    }
}

pub type ContractResult<T> = Result<T, ContractError>;
