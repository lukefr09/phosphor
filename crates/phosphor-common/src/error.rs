use thiserror::Error;

#[derive(Error, Debug)]
pub enum PhosphorError {
    #[error("PTY error: {0}")]
    Pty(String),
    
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Parse error: {0}")]
    Parse(String),
    
    #[error("Terminal state error: {0}")]
    State(String),
    
    #[error("Event system error: {0}")]
    Event(String),
    
    #[error("Configuration error: {0}")]
    Config(String),
    
    #[error("Platform error: {0}")]
    Platform(String),
}

pub type Result<T> = std::result::Result<T, PhosphorError>;