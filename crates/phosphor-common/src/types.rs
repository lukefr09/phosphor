use serde::{Deserialize, Serialize};

/// Terminal dimensions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Size {
    pub rows: u16,
    pub cols: u16,
}

impl Size {
    pub fn new(cols: u16, rows: u16) -> Self {
        Self { rows, cols }
    }
}

/// Cursor position (0-indexed)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Position {
    pub row: u16,
    pub col: u16,
}

impl Position {
    pub fn new(row: u16, col: u16) -> Self {
        Self { row, col }
    }
}

/// Character cell in the terminal
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cell {
    pub ch: char,
    pub attrs: CellAttributes,
}

impl Cell {
    pub fn new(ch: char) -> Self {
        Self {
            ch,
            attrs: CellAttributes::default(),
        }
    }

    pub fn with_attrs(ch: char, attrs: CellAttributes) -> Self {
        Self { ch, attrs }
    }

    pub fn blank() -> Self {
        Self::new(' ')
    }
}

impl Default for Cell {
    fn default() -> Self {
        Self::blank()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct CellAttributes {
    pub fg_color: Option<Color>,
    pub bg_color: Option<Color>,
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    pub strikethrough: bool,
    pub reverse: bool,
    pub hidden: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Color {
    Rgb(u8, u8, u8),
    Indexed(u8),
    Default,
}

/// Terminal mode flags
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct TerminalMode {
    pub echo: bool,
    pub raw: bool,
    pub line_wrap: bool,
    pub cursor_visible: bool,
}

/// Terminal state snapshot for serialization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminalSnapshot {
    pub size: Size,
    pub cursor: Position,
    pub mode: TerminalMode,
}