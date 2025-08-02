use phosphor_common::types::{
    Cell, Position, Size, TerminalMode, TerminalSnapshot, 
    CellAttributes, Color, CursorStyle, AttributeFlags
};
use phosphor_common::traits::Mode;
use tracing::{debug, instrument};

use super::buffer::{ScreenBuffer, ScrollbackBuffer};
use super::cursor::Cursor;

/// Terminal state machine that manages the display buffer and cursor
pub struct TerminalState {
    size: Size,
    cursor: Cursor,
    saved_cursor: Option<Cursor>,
    screen_buffer: ScreenBuffer,
    alternate_buffer: Option<ScreenBuffer>,
    scrollback_buffer: ScrollbackBuffer,
    mode: TerminalMode,
    cursor_style: CursorStyle,
    active_attributes: CellAttributes,
    color_palette: Vec<Color>,
    tab_stops: Vec<u16>,
}

impl TerminalState {
    /// Create a new terminal state with the given size
    pub fn new(size: Size) -> Self {
        debug!("Creating terminal state with size {:?}", size);
        Self {
            size,
            cursor: Cursor::new(),
            saved_cursor: None,
            screen_buffer: ScreenBuffer::new(size),
            alternate_buffer: None,
            scrollback_buffer: ScrollbackBuffer::new(10_000), // 10k lines
            mode: TerminalMode::default(),
            cursor_style: CursorStyle::default(),
            active_attributes: CellAttributes::default(),
            color_palette: Self::default_palette(),
            tab_stops: Self::default_tab_stops(size.cols),
        }
    }
    
    /// Create the default 256-color palette
    fn default_palette() -> Vec<Color> {
        let mut palette = Vec::with_capacity(256);
        
        // 0-15: Basic 16 colors
        for i in 0..16 {
            palette.push(Color::from_ansi(i));
        }
        
        // 16-231: 6x6x6 color cube
        for r in 0..6 {
            for g in 0..6 {
                for b in 0..6 {
                    let red = if r == 0 { 0 } else { 55 + r * 40 };
                    let green = if g == 0 { 0 } else { 55 + g * 40 };
                    let blue = if b == 0 { 0 } else { 55 + b * 40 };
                    palette.push(Color::Rgb(red, green, blue));
                }
            }
        }
        
        // 232-255: Grayscale
        for i in 0..24 {
            let gray = 8 + i * 10;
            palette.push(Color::Rgb(gray, gray, gray));
        }
        
        palette
    }
    
    /// Create default tab stops (every 8 columns)
    fn default_tab_stops(cols: u16) -> Vec<u16> {
        (0..cols).step_by(8).collect()
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
                // Skip if terminal has no size
                if self.size.rows == 0 || self.size.cols == 0 {
                    return;
                }
                
                // Check if cursor is out of bounds and scroll if needed
                if self.cursor.position().row >= self.size.rows {
                    self.scroll_up();
                    self.cursor.set_row(self.size.rows.saturating_sub(1));
                }
                
                // Write character at cursor position with current attributes
                let pos = self.cursor.position();
                let cell = Cell::with_attrs(ch, self.active_attributes);
                self.screen_buffer.set_cell(pos, cell);
                
                // Advance cursor
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
    
    /// Set the active text attributes
    pub fn set_attributes(&mut self, attrs: CellAttributes) {
        self.active_attributes = attrs;
    }
    
    /// Get the active text attributes
    pub fn attributes(&self) -> &CellAttributes {
        &self.active_attributes
    }
    
    /// Set a specific attribute flag
    pub fn set_attribute_flag(&mut self, flag: AttributeFlags, enabled: bool) {
        if enabled {
            self.active_attributes.flags.insert(flag);
        } else {
            self.active_attributes.flags.remove(flag);
        }
    }
    
    /// Set foreground color
    pub fn set_foreground_color(&mut self, color: Color) {
        self.active_attributes.fg_color = color;
    }
    
    /// Set background color
    pub fn set_background_color(&mut self, color: Color) {
        self.active_attributes.bg_color = color;
    }
    
    /// Reset all attributes to default
    pub fn reset_attributes(&mut self) {
        self.active_attributes = CellAttributes::default();
    }
    
    /// Advance cursor position after writing a character
    fn advance_cursor(&mut self) {
        // Skip if terminal has no size
        if self.size.rows == 0 || self.size.cols == 0 {
            return;
        }
        
        self.cursor.move_right(1);
        
        // Check for line wrap
        if self.cursor.position().col >= self.size.cols {
            if self.mode.contains(TerminalMode::LINE_WRAP) {
                self.cursor.set_column(0);
                self.cursor.move_down(1);
                
                // Check if we need to scroll
                if self.cursor.position().row >= self.size.rows {
                    self.scroll_up();
                    self.cursor.set_row(self.size.rows.saturating_sub(1));
                }
            } else {
                // Stay at the last column
                self.cursor.set_column(self.size.cols.saturating_sub(1));
            }
        }
    }
    
    /// Handle newline
    fn new_line(&mut self) {
        debug!("New line at cursor position {:?}", self.cursor.position());
        self.cursor.move_down(1);
        
        // Allow cursor to be on virtual row for proper newline handling
        // Scrolling only happens when writing text to out-of-bounds position
    }
    
    /// Handle carriage return
    fn carriage_return(&mut self) {
        debug!("Carriage return");
        self.cursor.set_column(0);
    }
    
    /// Perform a tab operation
    fn tab(&mut self) {
        let current_col = self.cursor.position().col;
        // Find next tab stop
        let next_tab = self.tab_stops.iter()
            .find(|&&stop| stop > current_col)
            .copied()
            .unwrap_or(self.size.cols - 1);
        self.cursor.set_column(next_tab);
    }
    
    /// Set a tab stop at current position
    pub fn set_tab_stop(&mut self) {
        let col = self.cursor.position().col;
        if !self.tab_stops.contains(&col) {
            self.tab_stops.push(col);
            self.tab_stops.sort();
        }
    }
    
    /// Clear tab stop at current position
    pub fn clear_tab_stop(&mut self) {
        let col = self.cursor.position().col;
        self.tab_stops.retain(|&stop| stop != col);
    }
    
    /// Clear all tab stops
    pub fn clear_all_tab_stops(&mut self) {
        self.tab_stops.clear();
    }
    
    /// Handle backspace
    fn backspace(&mut self) {
        self.cursor.saturating_left();
        self.advance_cursor();
        let cell = Cell::with_attrs(' ', self.active_attributes);
        self.screen_buffer.set_cell(self.cursor.position(), cell);
        self.cursor.saturating_left();
    }
    
    /// Scroll the terminal up by one line
    pub fn scroll_up(&mut self) {
        debug!("Scrolling up");
        
        // Move the first line to scrollback
        if let Some(line) = self.screen_buffer.remove_top_line() {
            self.scrollback_buffer.push(line);
        }
        
        // Add a new blank line at the bottom
        self.screen_buffer.add_blank_line();
    }
    
    /// Resize the terminal
    pub fn resize(&mut self, new_size: Size) {
        debug!("Resizing terminal from {:?} to {:?}", self.size, new_size);
        
        self.size = new_size;
        self.screen_buffer.resize(new_size);
        
        // Update tab stops for new width
        self.tab_stops = Self::default_tab_stops(new_size.cols);
        
        // Clamp cursor position
        let pos = self.cursor.position();
        self.cursor.set_position(Position::new(
            pos.row.min(new_size.rows.saturating_sub(1)),
            pos.col.min(new_size.cols.saturating_sub(1)),
        ));
    }
    
    /// Get the cursor position
    pub fn cursor_position(&self) -> Position {
        // Clamp position for external callers
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
    
    /// Get a mutable reference to the screen buffer
    pub fn screen_buffer_mut(&mut self) -> &mut ScreenBuffer {
        &mut self.screen_buffer
    }
    
    /// Get a mutable reference to the scrollback buffer
    pub fn scrollback_buffer_mut(&mut self) -> &mut ScrollbackBuffer {
        &mut self.scrollback_buffer
    }
    
    /// Get a mutable reference to the cursor
    pub fn cursor_mut(&mut self) -> &mut Cursor {
        &mut self.cursor
    }
    
    /// Set cursor position
    pub fn set_cursor_position(&mut self, pos: Position) {
        self.cursor.set_position(pos);
    }
    
    /// Set underline color
    pub fn set_underline_color(&mut self, color: Option<Color>) {
        self.active_attributes.underline_color = color;
    }
    
    /// Scroll down (reverse scroll)
    pub fn scroll_down(&mut self) {
        debug!("Scrolling down");
        // Insert blank line at top
        self.screen_buffer.insert_blank_line(0);
        // Remove bottom line
        self.screen_buffer.remove_bottom_line();
    }
    
    /// Set a terminal mode flag
    pub fn set_mode_flag(&mut self, mode: Mode, enabled: bool) {
        match mode {
            Mode::Insert => {
                if enabled {
                    self.mode.insert(TerminalMode::INSERT_MODE);
                } else {
                    self.mode.remove(TerminalMode::INSERT_MODE);
                }
            }
            Mode::AutoWrap => {
                if enabled {
                    self.mode.insert(TerminalMode::LINE_WRAP);
                } else {
                    self.mode.remove(TerminalMode::LINE_WRAP);
                }
            }
            Mode::BracketedPaste => {
                if enabled {
                    self.mode.insert(TerminalMode::BRACKETED_PASTE);
                } else {
                    self.mode.remove(TerminalMode::BRACKETED_PASTE);
                }
            }
            Mode::FocusReporting => {
                if enabled {
                    self.mode.insert(TerminalMode::FOCUS_REPORTING);
                } else {
                    self.mode.remove(TerminalMode::FOCUS_REPORTING);
                }
            }
            Mode::MouseReporting => {
                if enabled {
                    self.mode.insert(TerminalMode::MOUSE_REPORTING);
                } else {
                    self.mode.remove(TerminalMode::MOUSE_REPORTING);
                }
            }
            Mode::ApplicationCursor => {
                if enabled {
                    self.mode.insert(TerminalMode::APPLICATION_CURSOR);
                } else {
                    self.mode.remove(TerminalMode::APPLICATION_CURSOR);
                }
            }
            Mode::ApplicationKeypad => {
                if enabled {
                    self.mode.insert(TerminalMode::APPLICATION_KEYPAD);
                } else {
                    self.mode.remove(TerminalMode::APPLICATION_KEYPAD);
                }
            }
            Mode::OriginMode => {
                if enabled {
                    self.mode.insert(TerminalMode::ORIGIN_MODE);
                } else {
                    self.mode.remove(TerminalMode::ORIGIN_MODE);
                }
            }
            _ => {
                debug!("Unhandled mode flag: {:?}", mode);
            }
        }
    }
    
    /// Get the terminal mode
    pub fn mode(&self) -> TerminalMode {
        self.mode
    }
    
    /// Set terminal mode
    pub fn set_mode(&mut self, mode: TerminalMode) {
        self.mode = mode;
    }
    
    /// Enable alternate screen buffer
    pub fn enable_alternate_screen(&mut self) {
        if self.alternate_buffer.is_none() {
            let alt_buffer = ScreenBuffer::new(self.size);
            self.alternate_buffer = Some(std::mem::replace(&mut self.screen_buffer, alt_buffer));
            self.mode.insert(TerminalMode::ALTERNATE_SCREEN);
        }
    }
    
    /// Disable alternate screen buffer
    pub fn disable_alternate_screen(&mut self) {
        if let Some(main_buffer) = self.alternate_buffer.take() {
            self.screen_buffer = main_buffer;
            self.mode.remove(TerminalMode::ALTERNATE_SCREEN);
        }
    }
    
    /// Save cursor position and attributes
    pub fn save_cursor(&mut self) {
        self.saved_cursor = Some(self.cursor.clone());
    }
    
    /// Restore cursor position and attributes
    pub fn restore_cursor(&mut self) {
        if let Some(saved) = self.saved_cursor.take() {
            self.cursor = saved;
        }
    }
    
    /// Set cursor style
    pub fn set_cursor_style(&mut self, style: CursorStyle) {
        self.cursor_style = style;
    }
    
    /// Get cursor style
    pub fn cursor_style(&self) -> CursorStyle {
        self.cursor_style
    }
    
    /// Set cursor visibility
    pub fn set_cursor_visible(&mut self, visible: bool) {
        if visible {
            self.mode.insert(TerminalMode::CURSOR_VISIBLE);
        } else {
            self.mode.remove(TerminalMode::CURSOR_VISIBLE);
        }
    }
    
    /// Get a snapshot of the terminal state
    pub fn snapshot(&self) -> TerminalSnapshot {
        TerminalSnapshot {
            size: self.size,
            cursor: self.cursor.position(),
            mode: self.mode,
            cursor_style: self.cursor_style,
            active_attributes: self.active_attributes,
            alternate_screen_active: self.alternate_buffer.is_some(),
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