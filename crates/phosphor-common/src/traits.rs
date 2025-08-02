use crate::error::Result;
use crate::types::{Position, Size, TerminalSnapshot, Color};
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
    Csi(CsiSequence),
    Osc(OscSequence),
    Esc(EscSequence),
}

#[derive(Debug, Clone)]
pub enum ControlEvent {
    NewLine,
    CarriageReturn,
    Tab,
    Backspace,
    Clear,
    Bell,
    FormFeed,
    VerticalTab,
}

/// Control Sequence Introducer (CSI) sequences
#[derive(Debug, Clone)]
pub enum CsiSequence {
    // Cursor movement
    CursorUp(u16),
    CursorDown(u16),
    CursorForward(u16),
    CursorBack(u16),
    CursorPosition { row: u16, col: u16 },
    CursorColumn(u16),
    CursorNextLine(u16),
    CursorPreviousLine(u16),
    
    // Screen manipulation
    EraseDisplay(EraseMode),
    EraseLine(EraseMode),
    ScrollUp(u16),
    ScrollDown(u16),
    
    // Text attributes
    SetGraphicsRendition(Vec<SgrParameter>),
    
    // Cursor visibility
    ShowCursor,
    HideCursor,
    
    // Modes
    SetMode(Vec<Mode>),
    ResetMode(Vec<Mode>),
    
    // Device status
    DeviceStatusReport,
    CursorPositionReport,
    
    // Save/Restore cursor
    SaveCursor,
    RestoreCursor,
}

/// Operating System Command (OSC) sequences
#[derive(Debug, Clone)]
pub enum OscSequence {
    SetTitle(String),
    SetIcon(String),
    SetHyperlink { id: Option<String>, uri: String },
    ResetHyperlink,
    SetColor { index: u8, color: Color },
    ResetColor(u8),
    Clipboard { clipboard: ClipboardType, data: String },
}

/// ESC sequences (without CSI)
#[derive(Debug, Clone)]
pub enum EscSequence {
    Index,                    // Move cursor down one line
    NextLine,                 // Move to beginning of next line
    TabSet,                   // Set tab stop at current position
    ReverseIndex,             // Move cursor up one line
    KeypadApplicationMode,    // DECKPAM
    KeypadNumericMode,        // DECKPNM
    SaveCursor,               // DECSC
    RestoreCursor,            // DECRC
    Reset,                    // RIS - Reset to Initial State
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EraseMode {
    Below,      // From cursor to end
    Above,      // From beginning to cursor
    All,        // Entire display/line
    Saved,      // Erase saved lines (xterm)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SgrParameter {
    Reset,
    Bold,
    Dim,
    Italic,
    Underline,
    Blink,
    Reverse,
    Hidden,
    Strikethrough,
    
    NoBold,
    NoDim,
    NoItalic,
    NoUnderline,
    NoBlink,
    NoReverse,
    NoHidden,
    NoStrikethrough,
    
    Foreground(Color),
    Background(Color),
    UnderlineColor(Color),
    
    DefaultForeground,
    DefaultBackground,
    DefaultUnderlineColor,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    // ANSI modes
    KeyboardAction,           // KAM
    Insert,                   // IRM
    SendReceive,              // SRM
    LineFeed,                 // LNM
    
    // DEC private modes
    ApplicationCursor,        // DECCKM
    ApplicationKeypad,        // DECKPAM
    ColumnMode,               // DECCOLM
    ScrollMode,               // DECSCLM
    ScreenMode,               // DECSCNM
    OriginMode,               // DECOM
    AutoWrap,                 // DECAWM
    AutoRepeat,               // DECARM
    MouseReporting,           // Various mouse modes
    CursorVisible,            // DECTCEM
    AlternateScreen,          // Alternate screen buffer
    BracketedPaste,           // Bracketed paste mode
    FocusReporting,           // Focus in/out reporting
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClipboardType {
    Clipboard,
    Primary,
    Secondary,
}