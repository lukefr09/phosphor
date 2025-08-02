use phosphor_common::types::Size;

/// Commands that can be sent to the terminal
#[derive(Debug, Clone)]
pub enum Command {
    /// Write data to the PTY
    Write(Vec<u8>),
    
    /// Resize the terminal
    Resize(Size),
    
    /// Close the terminal
    Close,
}

/// Events emitted by the terminal
#[derive(Debug, Clone)]
pub enum Event {
    /// New output data available from PTY
    OutputReady(Vec<u8>),
    
    /// Terminal state has changed
    StateChanged,
    
    /// Terminal was resized
    Resized(Size),
    
    /// Terminal closed
    Closed,
    
    /// Error occurred
    Error(String),
}