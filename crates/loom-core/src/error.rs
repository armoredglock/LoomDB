use thiserror::Error;

#[derive(Error, Debug)]
pub enum LoomError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Corrupted record detected: checksum mismatch in {0}")]
    ChecksumMismatch(String),

    #[error("Key not found")]
    KeyNotFound,

    #[error("Transaction conflict: concurrent write detected")]
    TransactionConflict,

    #[error("Transaction is already committed or aborted")]
    TransactionClosed,

    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    #[error("Query parse error: {0}")]
    ParseError(String),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Internal engine error: {0}")]
    Internal(String),
}

pub type LoomResult<T> = Result<T, LoomError>;
