use thiserror::Error;

#[derive(Debug, Error)]
pub enum SyspulseError {
    #[error("Daemon '{0}' not found")]
    DaemonNotFound(String),
    #[error("Daemon '{0}' already exists")]
    DaemonAlreadyExists(String),
    #[error("Invalid state transition from {from:?} to {to:?}")]
    InvalidStateTransition { from: String, to: String },
    #[error("Process error: {0}")]
    Process(String),
    #[error("Health check failed: {0}")]
    HealthCheck(String),
    #[error("IPC error: {0}")]
    Ipc(String),
    #[error("Registry error: {0}")]
    Registry(String),
    #[error("Config error: {0}")]
    Config(String),
    #[error("Scheduler error: {0}")]
    Scheduler(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("Database error: {0}")]
    Database(String),
    #[error("Timeout after {0:?}")]
    Timeout(std::time::Duration),
}

pub type Result<T> = std::result::Result<T, SyspulseError>;
