use crate::error::Result;
use crate::types::{Position, Size, TerminalSnapshot};
use async_trait::async_trait;

/// Trait for terminal frontends (GUI frameworks)
#[async_trait]
pub trait TerminalFrontend: Send + Sync {
    /// Update the display with new terminal state
    async fn update(&mut self, snapshot: &TerminalSnapshot) -> Result<()>;
    
    /// Handle resize events
    async fn resize(&mut self, size: Size) -> Result<()>;
    
    /// Set cursor position
    async fn set_cursor(&mut self, position: Position) -> Result<()>;
    
    /// Refresh the display
    async fn refresh(&mut self) -> Result<()>;
}

/// Trait for terminal backends (PTY, parser, etc)
#[async_trait]
pub trait TerminalBackend: Send + Sync {
    /// Write data to the terminal
    async fn write(&mut self, data: &[u8]) -> Result<usize>;
    
    /// Read data from the terminal
    async fn read(&mut self, buf: &mut [u8]) -> Result<usize>;
    
    /// Resize the terminal
    async fn resize(&mut self, size: Size) -> Result<()>;
    
    /// Check if the backend is still alive
    async fn is_alive(&self) -> bool;
}

/// Trait for terminal parsers
pub trait TerminalParser: Send + Sync {
    /// Parse input data and return parsed events
    fn parse(&mut self, data: &[u8]) -> Vec<ParsedEvent>;
}

/// Events produced by the parser
#[derive(Debug, Clone)]
pub enum ParsedEvent {
    Text(String),
    Control(ControlEvent),
}

#[derive(Debug, Clone)]
pub enum ControlEvent {
    NewLine,
    CarriageReturn,
    Tab,
    Backspace,
    Clear,
    // More to be added in Phase 2
}