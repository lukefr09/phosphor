use phosphor_common::traits::{
    ParsedEvent, ControlEvent, CsiSequence, OscSequence, EscSequence,
    EraseMode, SgrParameter, Mode
};
use phosphor_common::types::{Position, Color, AttributeFlags};
use tracing::{debug, trace};

use crate::terminal::TerminalState;

/// ANSI escape sequence processor
pub struct AnsiProcessor;

impl AnsiProcessor {
    /// Process a parsed event and apply it to the terminal state
    pub fn process_event(state: &mut TerminalState, event: ParsedEvent) {
        match event {
            ParsedEvent::Text(text) => {
                trace!("Processing text: {:?}", text);
                state.write_str(&text);
            }
            ParsedEvent::Control(control) => {
                Self::process_control(state, control);
            }
            ParsedEvent::Csi(csi) => {
                Self::process_csi(state, csi);
            }
            ParsedEvent::Osc(osc) => {
                Self::process_osc(state, osc);
            }
            ParsedEvent::Esc(esc) => {
                Self::process_esc(state, esc);
            }
        }
    }
    
    fn process_control(state: &mut TerminalState, control: ControlEvent) {
        trace!("Processing control: {:?}", control);
        match control {
            ControlEvent::NewLine => state.write_char('\n'),
            ControlEvent::CarriageReturn => state.write_char('\r'),
            ControlEvent::Tab => state.write_char('\t'),
            ControlEvent::Backspace => state.write_char('\x08'),
            ControlEvent::Bell => {
                // TODO: Trigger bell event
                debug!("Bell");
            }
            ControlEvent::FormFeed => {
                // Form feed - often treated as clear screen
                Self::clear_screen(state, EraseMode::All);
            }
            ControlEvent::VerticalTab => {
                // Vertical tab - usually treated as newline
                state.write_char('\n');
            }
            ControlEvent::Clear => {
                Self::clear_screen(state, EraseMode::All);
            }
        }
    }
    
    fn process_csi(state: &mut TerminalState, csi: CsiSequence) {
        trace!("Processing CSI: {:?}", csi);
        match csi {
            // Cursor movement
            CsiSequence::CursorUp(n) => {
                state.cursor_mut().move_up(n);
            }
            CsiSequence::CursorDown(n) => {
                state.cursor_mut().move_down(n);
            }
            CsiSequence::CursorForward(n) => {
                state.cursor_mut().move_right(n);
            }
            CsiSequence::CursorBack(n) => {
                state.cursor_mut().move_left(n);
            }
            CsiSequence::CursorPosition { row, col } => {
                // ANSI uses 1-based indexing
                let pos = Position::new(
                    row.saturating_sub(1),
                    col.saturating_sub(1),
                );
                state.set_cursor_position(pos);
            }
            CsiSequence::CursorColumn(col) => {
                // ANSI uses 1-based indexing
                state.cursor_mut().set_column(col.saturating_sub(1));
            }
            CsiSequence::CursorNextLine(n) => {
                state.cursor_mut().set_column(0);
                state.cursor_mut().move_down(n);
            }
            CsiSequence::CursorPreviousLine(n) => {
                state.cursor_mut().set_column(0);
                state.cursor_mut().move_up(n);
            }
            
            // Screen manipulation
            CsiSequence::EraseDisplay(mode) => {
                Self::clear_screen(state, mode);
            }
            CsiSequence::EraseLine(mode) => {
                Self::clear_line(state, mode);
            }
            CsiSequence::ScrollUp(n) => {
                for _ in 0..n {
                    state.scroll_up();
                }
            }
            CsiSequence::ScrollDown(n) => {
                for _ in 0..n {
                    state.scroll_down();
                }
            }
            
            // Text attributes
            CsiSequence::SetGraphicsRendition(params) => {
                for param in params {
                    Self::apply_sgr(state, param);
                }
            }
            
            // Cursor visibility
            CsiSequence::ShowCursor => {
                state.set_cursor_visible(true);
            }
            CsiSequence::HideCursor => {
                state.set_cursor_visible(false);
            }
            
            // Modes
            CsiSequence::SetMode(modes) => {
                for mode in modes {
                    Self::set_mode(state, mode, true);
                }
            }
            CsiSequence::ResetMode(modes) => {
                for mode in modes {
                    Self::set_mode(state, mode, false);
                }
            }
            
            // Save/Restore cursor
            CsiSequence::SaveCursor => {
                state.save_cursor();
            }
            CsiSequence::RestoreCursor => {
                state.restore_cursor();
            }
            
            // Device status
            CsiSequence::DeviceStatusReport => {
                // TODO: Send response
                debug!("Device status report requested");
            }
            CsiSequence::CursorPositionReport => {
                // TODO: Send cursor position
                debug!("Cursor position report requested");
            }
        }
    }
    
    fn process_osc(_state: &mut TerminalState, osc: OscSequence) {
        trace!("Processing OSC: {:?}", osc);
        match osc {
            OscSequence::SetTitle(title) => {
                // TODO: Set window title
                debug!("Set title: {}", title);
            }
            OscSequence::SetIcon(icon) => {
                // TODO: Set window icon
                debug!("Set icon: {}", icon);
            }
            OscSequence::SetHyperlink { id, uri } => {
                // TODO: Store hyperlink info
                debug!("Set hyperlink: id={:?}, uri={}", id, uri);
            }
            OscSequence::ResetHyperlink => {
                // TODO: Clear hyperlink
                debug!("Reset hyperlink");
            }
            OscSequence::SetColor { index, color } => {
                // TODO: Update color palette
                debug!("Set color {}: {:?}", index, color);
            }
            OscSequence::ResetColor(index) => {
                // TODO: Reset color to default
                debug!("Reset color {}", index);
            }
            OscSequence::Clipboard { clipboard, data } => {
                // TODO: Handle clipboard operations
                debug!("Clipboard {:?}: {}", clipboard, data);
            }
        }
    }
    
    fn process_esc(state: &mut TerminalState, esc: EscSequence) {
        trace!("Processing ESC: {:?}", esc);
        match esc {
            EscSequence::Index => {
                // Move cursor down one line, scroll if at bottom
                state.cursor_mut().move_down(1);
                if state.cursor_position().row >= state.size().rows - 1 {
                    state.scroll_up();
                }
            }
            EscSequence::NextLine => {
                state.cursor_mut().set_column(0);
                state.cursor_mut().move_down(1);
            }
            EscSequence::TabSet => {
                state.set_tab_stop();
            }
            EscSequence::ReverseIndex => {
                // Move cursor up one line, scroll if at top
                if state.cursor_position().row == 0 {
                    state.scroll_down();
                } else {
                    state.cursor_mut().move_up(1);
                }
            }
            EscSequence::KeypadApplicationMode => {
                state.set_mode_flag(Mode::ApplicationKeypad, true);
            }
            EscSequence::KeypadNumericMode => {
                state.set_mode_flag(Mode::ApplicationKeypad, false);
            }
            EscSequence::SaveCursor => {
                state.save_cursor();
            }
            EscSequence::RestoreCursor => {
                state.restore_cursor();
            }
            EscSequence::Reset => {
                // Reset terminal to initial state
                *state = TerminalState::new(state.size());
            }
        }
    }
    
    fn apply_sgr(state: &mut TerminalState, param: SgrParameter) {
        match param {
            SgrParameter::Reset => {
                state.reset_attributes();
            }
            SgrParameter::Bold => {
                state.set_attribute_flag(AttributeFlags::BOLD, true);
            }
            SgrParameter::Dim => {
                state.set_attribute_flag(AttributeFlags::DIM, true);
            }
            SgrParameter::Italic => {
                state.set_attribute_flag(AttributeFlags::ITALIC, true);
            }
            SgrParameter::Underline => {
                state.set_attribute_flag(AttributeFlags::UNDERLINE, true);
            }
            SgrParameter::Blink => {
                state.set_attribute_flag(AttributeFlags::BLINK_SLOW, true);
            }
            SgrParameter::Reverse => {
                state.set_attribute_flag(AttributeFlags::REVERSE, true);
            }
            SgrParameter::Hidden => {
                state.set_attribute_flag(AttributeFlags::HIDDEN, true);
            }
            SgrParameter::Strikethrough => {
                state.set_attribute_flag(AttributeFlags::STRIKETHROUGH, true);
            }
            
            SgrParameter::NoBold => {
                state.set_attribute_flag(AttributeFlags::BOLD, false);
                state.set_attribute_flag(AttributeFlags::DIM, false);
            }
            SgrParameter::NoDim => {
                state.set_attribute_flag(AttributeFlags::DIM, false);
            }
            SgrParameter::NoItalic => {
                state.set_attribute_flag(AttributeFlags::ITALIC, false);
            }
            SgrParameter::NoUnderline => {
                state.set_attribute_flag(AttributeFlags::UNDERLINE, false);
            }
            SgrParameter::NoBlink => {
                state.set_attribute_flag(AttributeFlags::BLINK_SLOW, false);
                state.set_attribute_flag(AttributeFlags::BLINK_FAST, false);
            }
            SgrParameter::NoReverse => {
                state.set_attribute_flag(AttributeFlags::REVERSE, false);
            }
            SgrParameter::NoHidden => {
                state.set_attribute_flag(AttributeFlags::HIDDEN, false);
            }
            SgrParameter::NoStrikethrough => {
                state.set_attribute_flag(AttributeFlags::STRIKETHROUGH, false);
            }
            
            SgrParameter::Foreground(color) => {
                state.set_foreground_color(color);
            }
            SgrParameter::Background(color) => {
                state.set_background_color(color);
            }
            SgrParameter::UnderlineColor(color) => {
                state.set_underline_color(Some(color));
            }
            
            SgrParameter::DefaultForeground => {
                state.set_foreground_color(Color::Default);
            }
            SgrParameter::DefaultBackground => {
                state.set_background_color(Color::Default);
            }
            SgrParameter::DefaultUnderlineColor => {
                state.set_underline_color(None);
            }
        }
    }
    
    fn clear_screen(state: &mut TerminalState, mode: EraseMode) {
        let size = state.size();
        let cursor_pos = state.cursor_position();
        
        match mode {
            EraseMode::Below => {
                // Clear from cursor to end of screen
                for row in cursor_pos.row..size.rows {
                    for col in 0..size.cols {
                        if row == cursor_pos.row && col < cursor_pos.col {
                            continue;
                        }
                        state.screen_buffer_mut().clear_cell(Position::new(row, col));
                    }
                }
            }
            EraseMode::Above => {
                // Clear from beginning to cursor
                for row in 0..=cursor_pos.row {
                    for col in 0..size.cols {
                        if row == cursor_pos.row && col > cursor_pos.col {
                            break;
                        }
                        state.screen_buffer_mut().clear_cell(Position::new(row, col));
                    }
                }
            }
            EraseMode::All => {
                // Clear entire screen
                state.screen_buffer_mut().clear();
            }
            EraseMode::Saved => {
                // Clear saved lines (scrollback)
                state.scrollback_buffer_mut().clear();
            }
        }
    }
    
    fn clear_line(state: &mut TerminalState, mode: EraseMode) {
        let cursor_pos = state.cursor_position();
        let cols = state.size().cols;
        
        match mode {
            EraseMode::Below => {
                // Clear from cursor to end of line
                for col in cursor_pos.col..cols {
                    state.screen_buffer_mut().clear_cell(Position::new(cursor_pos.row, col));
                }
            }
            EraseMode::Above => {
                // Clear from beginning to cursor
                for col in 0..=cursor_pos.col {
                    state.screen_buffer_mut().clear_cell(Position::new(cursor_pos.row, col));
                }
            }
            EraseMode::All | EraseMode::Saved => {
                // Clear entire line
                for col in 0..cols {
                    state.screen_buffer_mut().clear_cell(Position::new(cursor_pos.row, col));
                }
            }
        }
    }
    
    fn set_mode(state: &mut TerminalState, mode: Mode, enabled: bool) {
        match mode {
            Mode::Insert => {
                state.set_mode_flag(Mode::Insert, enabled);
            }
            Mode::AutoWrap => {
                state.set_mode_flag(Mode::AutoWrap, enabled);
            }
            Mode::CursorVisible => {
                state.set_cursor_visible(enabled);
            }
            Mode::AlternateScreen => {
                if enabled {
                    state.enable_alternate_screen();
                } else {
                    state.disable_alternate_screen();
                }
            }
            Mode::BracketedPaste => {
                state.set_mode_flag(Mode::BracketedPaste, enabled);
            }
            Mode::FocusReporting => {
                state.set_mode_flag(Mode::FocusReporting, enabled);
            }
            Mode::MouseReporting => {
                state.set_mode_flag(Mode::MouseReporting, enabled);
            }
            Mode::ApplicationCursor => {
                state.set_mode_flag(Mode::ApplicationCursor, enabled);
            }
            Mode::OriginMode => {
                state.set_mode_flag(Mode::OriginMode, enabled);
            }
            _ => {
                debug!("Unhandled mode: {:?}", mode);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use phosphor_common::types::Size;
    use phosphor_parser::VteParser;
    use phosphor_common::traits::TerminalParser;
    
    #[test]
    fn test_cursor_movement() {
        let mut state = TerminalState::new(Size::new(80, 24));
        let mut parser = VteParser::new();
        
        // Move cursor to position 10,20
        let events = parser.parse(b"\x1b[10;20H");
        for event in events {
            AnsiProcessor::process_event(&mut state, event);
        }
        assert_eq!(state.cursor_position(), Position::new(9, 19)); // 0-indexed
        
        // Move cursor up 5
        let events = parser.parse(b"\x1b[5A");
        for event in events {
            AnsiProcessor::process_event(&mut state, event);
        }
        assert_eq!(state.cursor_position(), Position::new(4, 19));
    }
    
    #[test]
    fn test_colors() {
        let mut state = TerminalState::new(Size::new(80, 24));
        let mut parser = VteParser::new();
        
        // Set foreground to red, background to blue
        let events = parser.parse(b"\x1b[31;44m");
        for event in events {
            AnsiProcessor::process_event(&mut state, event);
        }
        
        let attrs = state.attributes();
        assert_eq!(attrs.fg_color, Color::Red);
        assert_eq!(attrs.bg_color, Color::Blue);
        
        // Reset
        let events = parser.parse(b"\x1b[0m");
        for event in events {
            AnsiProcessor::process_event(&mut state, event);
        }
        
        let attrs = state.attributes();
        assert_eq!(attrs.fg_color, Color::Default);
        assert_eq!(attrs.bg_color, Color::Default);
    }
    
    #[test]
    fn test_text_attributes() {
        let mut state = TerminalState::new(Size::new(80, 24));
        let mut parser = VteParser::new();
        
        // Bold, italic, underline
        let events = parser.parse(b"\x1b[1;3;4m");
        for event in events {
            AnsiProcessor::process_event(&mut state, event);
        }
        
        let attrs = state.attributes();
        assert!(attrs.flags.contains(AttributeFlags::BOLD));
        assert!(attrs.flags.contains(AttributeFlags::ITALIC));
        assert!(attrs.flags.contains(AttributeFlags::UNDERLINE));
    }
}