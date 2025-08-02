# Phase 2: Advanced Terminal Protocol Support

## Overview

Phase 2 transforms Phosphor Terminal from a basic text display into a fully-featured ANSI/VT-compatible terminal emulator. This phase implements comprehensive escape sequence parsing, color support, text attributes, and advanced terminal features required for modern command-line applications.

## Implemented Features

### 1. VTE Integration

Replaced the basic parser with the industry-standard `vte` crate, providing:

- **Robust Parsing**: Battle-tested escape sequence parsing used by major terminal emulators
- **Standards Compliance**: Full VT100/VT220 compatibility
- **Performance**: Optimized state machine for high-throughput parsing
- **Extensibility**: Support for modern terminal extensions

#### Implementation Details
```rust
// phosphor-parser/src/lib.rs
pub struct VteParser {
    parser: vte::Parser,
    performer: TerminalPerformer,
}

impl TerminalParser for VteParser {
    fn parse(&mut self, data: &[u8]) -> Vec<ParsedEvent> {
        // Process bytes through VTE state machine
        // Convert VTE events to our ParsedEvent types
    }
}
```

### 2. Color System

Implemented comprehensive color support:

#### Color Types
- **4-bit ANSI Colors**: Standard 16 colors (8 normal + 8 bright)
- **8-bit Indexed Colors**: 256-color palette
  - Colors 0-15: Standard ANSI colors
  - Colors 16-231: 6×6×6 RGB color cube
  - Colors 232-255: 24-step grayscale ramp
- **24-bit True Color**: Full RGB support (16.7 million colors)

#### Color Representation
```rust
pub enum Color {
    Default,
    Black, Red, Green, Yellow, Blue, Magenta, Cyan, White,
    BrightBlack, BrightRed, BrightGreen, BrightYellow,
    BrightBlue, BrightMagenta, BrightCyan, BrightWhite,
    Indexed(u8),      // 0-255 palette index
    Rgb(u8, u8, u8),  // True color
}
```

### 3. Text Attributes

Enhanced cell attributes with comprehensive text styling:

#### Attribute Flags
```rust
bitflags! {
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
```

### 4. ANSI Escape Sequences

#### CSI (Control Sequence Introducer) Sequences

**Cursor Movement**
- `ESC[{n}A` - Cursor Up
- `ESC[{n}B` - Cursor Down
- `ESC[{n}C` - Cursor Forward
- `ESC[{n}D` - Cursor Back
- `ESC[{row};{col}H` - Cursor Position
- `ESC[{n}G` - Cursor Horizontal Absolute
- `ESC[{n}E` - Cursor Next Line
- `ESC[{n}F` - Cursor Previous Line

**Screen Manipulation**
- `ESC[{n}J` - Erase Display (0=below, 1=above, 2=all, 3=saved)
- `ESC[{n}K` - Erase Line (0=right, 1=left, 2=all)
- `ESC[{n}S` - Scroll Up
- `ESC[{n}T` - Scroll Down

**Text Formatting (SGR)**
- `ESC[0m` - Reset all attributes
- `ESC[1m` - Bold
- `ESC[3m` - Italic
- `ESC[4m` - Underline
- `ESC[7m` - Reverse video
- `ESC[30-37m` - Foreground color
- `ESC[40-47m` - Background color
- `ESC[38;5;{n}m` - 256-color foreground
- `ESC[38;2;{r};{g};{b}m` - RGB foreground

**Cursor Visibility**
- `ESC[?25h` - Show cursor
- `ESC[?25l` - Hide cursor

#### OSC (Operating System Command) Sequences

- `OSC 0;{title} BEL` - Set window title
- `OSC 8;{params};{uri} BEL` - Hyperlink support
- `OSC 4;{index};{color} BEL` - Set color palette

#### ESC Sequences

- `ESC D` - Index (move down, scroll if needed)
- `ESC M` - Reverse Index
- `ESC E` - Next Line
- `ESC H` - Tab Set
- `ESC c` - Reset to Initial State
- `ESC 7` - Save Cursor (DECSC)
- `ESC 8` - Restore Cursor (DECRC)

### 5. Terminal Modes

Implemented terminal mode management with bitflags:

```rust
bitflags! {
    pub struct TerminalMode: u32 {
        const LINE_WRAP         = 1 << 2;
        const CURSOR_VISIBLE    = 1 << 3;
        const ALTERNATE_SCREEN  = 1 << 5;
        const BRACKETED_PASTE   = 1 << 6;
        const FOCUS_REPORTING   = 1 << 7;
        const MOUSE_REPORTING   = 1 << 8;
        const APPLICATION_CURSOR = 1 << 11;
        const APPLICATION_KEYPAD = 1 << 12;
        const ORIGIN_MODE       = 1 << 13;
        const INSERT_MODE       = 1 << 14;
    }
}
```

### 6. Alternate Screen Buffer

Full support for alternate screen switching used by applications like `vim`, `less`, and `htop`:

```rust
pub fn enable_alternate_screen(&mut self) {
    // Save main buffer
    // Switch to alternate buffer
    // Set ALTERNATE_SCREEN mode flag
}

pub fn disable_alternate_screen(&mut self) {
    // Restore main buffer
    // Clear ALTERNATE_SCREEN mode flag
}
```

### 7. Enhanced Terminal State

Extended the terminal state machine with:

- **Color Palette**: 256-color palette with defaults
- **Active Attributes**: Current text styling state
- **Cursor Styles**: Block, underline, bar (with blinking variants)
- **Tab Stops**: Configurable tab positions
- **Saved Cursor**: DECSC/DECRC support

### 8. ANSI Processor

Created a dedicated ANSI processor module that:

1. Receives parsed events from VTE
2. Applies state changes to the terminal
3. Handles complex sequences requiring multiple state updates
4. Manages mode transitions

```rust
pub struct AnsiProcessor;

impl AnsiProcessor {
    pub fn process_event(state: &mut TerminalState, event: ParsedEvent) {
        match event {
            ParsedEvent::Text(text) => { /* write text */ },
            ParsedEvent::Csi(csi) => { /* handle CSI sequence */ },
            ParsedEvent::Osc(osc) => { /* handle OSC sequence */ },
            ParsedEvent::Esc(esc) => { /* handle ESC sequence */ },
            ParsedEvent::Control(ctrl) => { /* handle control char */ },
        }
    }
}
```

## Testing

### Unit Tests

Comprehensive test coverage for:

- **Parser Tests**: Validate VTE integration and event generation
- **Color Tests**: 4-bit, 8-bit, and 24-bit color parsing
- **Attribute Tests**: Text styling combinations
- **Sequence Tests**: Cursor movement, screen manipulation
- **State Tests**: Mode changes, alternate screen

### Integration Tests

- Full ANSI sequence processing pipeline
- Terminal state consistency
- Edge cases and error handling

## Performance Characteristics

- **Parsing Speed**: VTE provides optimized byte-by-byte processing
- **Memory Usage**: Minimal overhead per cell (~24 bytes with attributes)
- **Color Lookups**: O(1) for all color types
- **Attribute Changes**: Bitflag operations for fast updates

## Usage Examples

### Basic Color Support
```bash
# 4-bit colors
echo -e "\033[31mRed text\033[0m"
echo -e "\033[42mGreen background\033[0m"

# 256 colors
echo -e "\033[38;5;208mOrange text\033[0m"

# True color
echo -e "\033[38;2;255;128;0mRGB Orange\033[0m"
```

### Text Attributes
```bash
# Combinations
echo -e "\033[1;4mBold and underlined\033[0m"
echo -e "\033[3;31mItalic red text\033[0m"
```

### Cursor Control
```bash
# Save position, move around, restore
echo -e "\033[s\033[10;20HMoved text\033[u"
```

## Compatibility

### Supported Applications
- Basic shells (bash, zsh, fish)
- Text editors (nano, vim with basic features)
- File managers (mc, ranger basics)
- System monitors (htop, top)
- Development tools (git, grep with colors)

### Known Limitations
- No graphics protocols yet (Sixel, iTerm2)
- No custom cursor shapes yet
- Mouse support not implemented
- Some advanced VT sequences pending

## Next Steps (Phase 3)

With ANSI support complete, Phase 3 will focus on:
1. GPU-accelerated rendering with WGPU
2. CRT shader effects
3. Font rendering with ligatures
4. Performance optimizations

## API Changes

### New Public APIs
```rust
// Terminal state extensions
impl TerminalState {
    pub fn set_attributes(&mut self, attrs: CellAttributes);
    pub fn set_foreground_color(&mut self, color: Color);
    pub fn set_background_color(&mut self, color: Color);
    pub fn enable_alternate_screen(&mut self);
    pub fn set_cursor_style(&mut self, style: CursorStyle);
}
```

### Breaking Changes
None - Phase 1 APIs remain compatible.

## Conclusion

Phase 2 successfully transforms Phosphor from a basic terminal to a fully-featured ANSI-compatible emulator. The VTE integration provides robust parsing, while the comprehensive color and attribute support enables rich terminal applications. The foundation is now ready for Phase 3's visual enhancements.