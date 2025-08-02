use phosphor_common::types::{Cell, Position, Size, TerminalMode, TerminalSnapshot};
use tracing::{debug, instrument};

use super::buffer::{ScreenBuffer, ScrollbackBuffer};
use super::cursor::Cursor;

/// Terminal state machine that manages the display buffer and cursor
pub struct TerminalState {
    size: Size,
    cursor: Cursor,
    screen_buffer: ScreenBuffer,
    scrollback_buffer: ScrollbackBuffer,
    mode: TerminalMode,
}

impl TerminalState {
    /// Create a new terminal state with the given size
    pub fn new(size: Size) -> Self {
        debug!("Creating terminal state with size {:?}", size);
        Self {
            size,
            cursor: Cursor::new(),
            screen_buffer: ScreenBuffer::new(size),
            scrollback_buffer: ScrollbackBuffer::new(10_000), // 10k lines
            mode: TerminalMode {
                echo: true,
                raw: false,
                line_wrap: true,
                cursor_visible: true,
            },
        }
    }
    
    /// Write a character to the terminal
    #[instrument(skip(self))]
    pub fn write_char(&mut self, ch: char) {
        match ch {
            '\n' => self.new_line(),
            '\r' => self.carriage_return(),
            '\t' => self.tab(),
            '\x08' => self.backspace(),
            '\x00' => {}, // Null character, ignore
            _ => {
                // Check if cursor is out of bounds and scroll if needed
                if self.cursor.position().row >= self.size.rows {
                    self.scroll_up();
                    self.cursor.set_row(self.size.rows - 1);
                }
                
                // Write character at cursor position
                let pos = self.cursor.position();
                self.screen_buffer.set_cell(pos, Cell::new(ch));
                self.advance_cursor();
            }
        }
    }
    
    /// Write a string to the terminal
    pub fn write_str(&mut self, s: &str) {
        for ch in s.chars() {
            self.write_char(ch);
        }
    }
    
    /// Move to the next line
    fn new_line(&mut self) {
        debug!("New line");
        self.cursor.move_down(1);
        
        // Note: We don't scroll here. Scrolling happens when we try to write
        // text on an out-of-bounds row. This allows the cursor to be positioned
        // on a "virtual" row below the screen until text is written there.
    }
    
    /// Move cursor to beginning of line
    fn carriage_return(&mut self) {
        debug!("Carriage return");
        self.cursor.set_col(0);
    }
    
    /// Move to next tab stop
    fn tab(&mut self) {
        // Move to next tab stop (every 8 columns)
        let current_col = self.cursor.position().col;
        let next_tab = ((current_col / 8) + 1) * 8;
        let new_col = next_tab.min(self.size.cols - 1);
        self.cursor.set_col(new_col);
    }
    
    /// Handle backspace
    fn backspace(&mut self) {
        if self.cursor.position().col > 0 {
            self.cursor.move_left(1);
        }
    }
    
    /// Advance cursor by one position
    fn advance_cursor(&mut self) {
        let pos = self.cursor.position();
        
        // Check if we need to wrap before moving
        if pos.col >= self.size.cols - 1 {
            // At the end of the line
            if self.mode.line_wrap {
                self.cursor.set_col(0);
                self.new_line();
            } else {
                // Stay at the end of the line
                self.cursor.set_col(self.size.cols - 1);
            }
        } else {
            // Normal advance
            self.cursor.move_right(1);
        }
    }
    
    /// Scroll the screen up by one line
    fn scroll_up(&mut self) {
        debug!("Scrolling up");
        // Move top line to scrollback
        if let Some(line) = self.screen_buffer.remove_top_line() {
            self.scrollback_buffer.push(line);
        }
        
        // Add blank line at bottom
        self.screen_buffer.add_blank_line();
    }
    
    /// Resize the terminal
    pub fn resize(&mut self, new_size: Size) {
        debug!("Resizing terminal from {:?} to {:?}", self.size, new_size);
        self.size = new_size;
        self.screen_buffer.resize(new_size);
        
        // Ensure cursor is within bounds
        let pos = self.cursor.position();
        if pos.row >= new_size.rows {
            self.cursor.set_row(new_size.rows - 1);
        }
        if pos.col >= new_size.cols {
            self.cursor.set_col(new_size.cols - 1);
        }
    }
    
    /// Get the current cursor position (clamped to screen bounds)
    pub fn cursor_position(&self) -> Position {
        let pos = self.cursor.position();
        Position::new(
            pos.row.min(self.size.rows.saturating_sub(1)),
            pos.col.min(self.size.cols.saturating_sub(1)),
        )
    }
    
    /// Get the terminal size
    pub fn size(&self) -> Size {
        self.size
    }
    
    /// Get a reference to the screen buffer
    pub fn screen_buffer(&self) -> &ScreenBuffer {
        &self.screen_buffer
    }
    
    /// Get a reference to the scrollback buffer
    pub fn scrollback_buffer(&self) -> &ScrollbackBuffer {
        &self.scrollback_buffer
    }
    
    /// Get the terminal mode
    pub fn mode(&self) -> TerminalMode {
        self.mode
    }
    
    /// Set cursor visibility
    pub fn set_cursor_visible(&mut self, visible: bool) {
        self.mode.cursor_visible = visible;
    }
    
    /// Get a snapshot of the terminal state
    pub fn snapshot(&self) -> TerminalSnapshot {
        TerminalSnapshot {
            size: self.size,
            cursor: self.cursor.position(),
            mode: self.mode,
        }
    }
    
    /// Ensure cursor is within bounds
    fn clamp_cursor(&mut self) {
        let pos = self.cursor.position();
        if pos.row >= self.size.rows {
            self.cursor.set_row(self.size.rows - 1);
        }
        if pos.col >= self.size.cols {
            self.cursor.set_col(self.size.cols - 1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_write_char() {
        let mut state = TerminalState::new(Size::new(80, 24));
        state.write_char('A');
        assert_eq!(state.cursor_position(), Position::new(0, 1));
        
        let cell = state.screen_buffer().get_cell(Position::new(0, 0));
        assert_eq!(cell.ch, 'A');
    }
    
    #[test]
    fn test_newline() {
        let mut state = TerminalState::new(Size::new(80, 24));
        state.write_char('\n');
        assert_eq!(state.cursor_position(), Position::new(1, 0));
    }
    
    #[test]
    fn test_carriage_return() {
        let mut state = TerminalState::new(Size::new(80, 24));
        state.write_str("Hello");
        state.write_char('\r');
        assert_eq!(state.cursor_position(), Position::new(0, 0));
    }
    
    #[test]
    fn test_line_wrap() {
        let mut state = TerminalState::new(Size::new(3, 24));
        state.write_str("ABCD");
        assert_eq!(state.cursor_position(), Position::new(1, 1));
    }
    
    #[test]
    fn test_tab() {
        let mut state = TerminalState::new(Size::new(80, 24));
        state.write_char('\t');
        assert_eq!(state.cursor_position(), Position::new(0, 8));
        
        state.write_char('X');
        state.write_char('\t');
        assert_eq!(state.cursor_position(), Position::new(0, 16));
    }
    
    #[test]
    fn test_scroll() {
        let mut state = TerminalState::new(Size::new(80, 3));
        
        // Fill the screen
        for i in 0..4 {
            state.write_str(&format!("Line {}\n", i));
        }
        
        // Should have scrolled
        assert_eq!(state.cursor_position().row, 2);
        assert_eq!(state.scrollback_buffer().len(), 1);
    }
    
    #[test]
    fn debug_scroll() {
        let mut state = TerminalState::new(Size::new(80, 3));
        
        println!("Initial: cursor={:?}, scrollback={}", 
                 state.cursor_position(), state.scrollback_buffer().len());
        
        for i in 0..4 {
            state.write_str(&format!("Line {}\n", i));
            println!("After Line {}: cursor={:?}, scrollback={}", 
                     i, state.cursor_position(), state.scrollback_buffer().len());
        }
    }
}