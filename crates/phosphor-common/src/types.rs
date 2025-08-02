use serde::{Deserialize, Serialize};
use bitflags::bitflags;

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
    pub hyperlink: Option<String>,
}

impl Cell {
    pub fn new(ch: char) -> Self {
        Self {
            ch,
            attrs: CellAttributes::default(),
            hyperlink: None,
        }
    }

    pub fn with_attrs(ch: char, attrs: CellAttributes) -> Self {
        Self { ch, attrs, hyperlink: None }
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

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    pub struct AttributeFlags: u16 {
        const BOLD          = 1 << 0;
        const ITALIC        = 1 << 1;
        const UNDERLINE     = 1 << 2;
        const STRIKETHROUGH = 1 << 3;
        const BLINK_SLOW    = 1 << 4;
        const BLINK_FAST    = 1 << 5;
        const REVERSE       = 1 << 6;
        const HIDDEN        = 1 << 7;
        const DIM           = 1 << 8;
        const DOUBLE_UNDERLINE = 1 << 9;
        const CURLY_UNDERLINE  = 1 << 10;
        const DOTTED_UNDERLINE = 1 << 11;
        const DASHED_UNDERLINE = 1 << 12;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct CellAttributes {
    pub fg_color: Color,
    pub bg_color: Color,
    pub flags: AttributeFlags,
    pub underline_color: Option<Color>,
}

impl Default for CellAttributes {
    fn default() -> Self {
        Self {
            fg_color: Color::Default,
            bg_color: Color::Default,
            flags: AttributeFlags::empty(),
            underline_color: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Color {
    Default,
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    White,
    BrightBlack,
    BrightRed,
    BrightGreen,
    BrightYellow,
    BrightBlue,
    BrightMagenta,
    BrightCyan,
    BrightWhite,
    Indexed(u8),      // 0-255
    Rgb(u8, u8, u8),  // True color
}

impl Color {
    /// Convert 4-bit ANSI color index to Color enum
    pub fn from_ansi(index: u8) -> Self {
        match index {
            0 => Color::Black,
            1 => Color::Red,
            2 => Color::Green,
            3 => Color::Yellow,
            4 => Color::Blue,
            5 => Color::Magenta,
            6 => Color::Cyan,
            7 => Color::White,
            8 => Color::BrightBlack,
            9 => Color::BrightRed,
            10 => Color::BrightGreen,
            11 => Color::BrightYellow,
            12 => Color::BrightBlue,
            13 => Color::BrightMagenta,
            14 => Color::BrightCyan,
            15 => Color::BrightWhite,
            _ => Color::Indexed(index),
        }
    }
}

/// Cursor style
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CursorStyle {
    Block,
    Underline,
    Bar,
    BlinkingBlock,
    BlinkingUnderline,
    BlinkingBar,
}

impl Default for CursorStyle {
    fn default() -> Self {
        Self::Block
    }
}

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    pub struct TerminalMode: u32 {
        const ECHO              = 1 << 0;
        const RAW               = 1 << 1;
        const LINE_WRAP         = 1 << 2;
        const CURSOR_VISIBLE    = 1 << 3;
        const CURSOR_BLINKING   = 1 << 4;
        const ALTERNATE_SCREEN  = 1 << 5;
        const BRACKETED_PASTE   = 1 << 6;
        const FOCUS_REPORTING   = 1 << 7;
        const MOUSE_REPORTING   = 1 << 8;
        const MOUSE_MOTION      = 1 << 9;
        const MOUSE_SGR         = 1 << 10;
        const APPLICATION_CURSOR = 1 << 11;
        const APPLICATION_KEYPAD = 1 << 12;
        const ORIGIN_MODE       = 1 << 13;
        const INSERT_MODE       = 1 << 14;
        const REVERSE_VIDEO     = 1 << 15;
    }
}

impl Default for TerminalMode {
    fn default() -> Self {
        Self::LINE_WRAP | Self::CURSOR_VISIBLE | Self::ECHO
    }
}

/// Terminal state snapshot for serialization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminalSnapshot {
    pub size: Size,
    pub cursor: Position,
    pub cursor_style: CursorStyle,
    pub mode: TerminalMode,
    pub active_attributes: CellAttributes,
    pub alternate_screen_active: bool,
}