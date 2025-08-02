use phosphor_common::types::Position;

/// Cursor state and operations
#[derive(Debug, Clone)]
pub struct Cursor {
    position: Position,
    saved_position: Option<Position>,
    visible: bool,
}

impl Cursor {
    /// Create a new cursor at the origin
    pub fn new() -> Self {
        Self {
            position: Position::new(0, 0),
            saved_position: None,
            visible: true,
        }
    }
    
    /// Get the current cursor position
    pub fn position(&self) -> Position {
        self.position
    }
    
    /// Set the cursor position
    pub fn set_position(&mut self, pos: Position) {
        self.position = pos;
    }
    
    /// Set the cursor row
    pub fn set_row(&mut self, row: u16) {
        self.position.row = row;
    }
    
    /// Set the cursor column
    pub fn set_col(&mut self, col: u16) {
        self.position.col = col;
    }
    
    /// Set the cursor column (alias for set_col)
    pub fn set_column(&mut self, col: u16) {
        self.position.col = col;
    }
    
    /// Move cursor up by n rows
    pub fn move_up(&mut self, n: u16) {
        self.position.row = self.position.row.saturating_sub(n);
    }
    
    /// Move cursor down by n rows
    pub fn move_down(&mut self, n: u16) {
        self.position.row = self.position.row.saturating_add(n);
    }
    
    /// Move cursor left by n columns
    pub fn move_left(&mut self, n: u16) {
        self.position.col = self.position.col.saturating_sub(n);
    }
    
    /// Move cursor right by n columns
    pub fn move_right(&mut self, n: u16) {
        self.position.col = self.position.col.saturating_add(n);
    }
    
    /// Move cursor left by 1 column (saturating)
    pub fn saturating_left(&mut self) {
        self.position.col = self.position.col.saturating_sub(1);
    }
    
    /// Save the current cursor position
    pub fn save(&mut self) {
        self.saved_position = Some(self.position);
    }
    
    /// Restore the saved cursor position
    pub fn restore(&mut self) {
        if let Some(pos) = self.saved_position {
            self.position = pos;
        }
    }
    
    /// Check if cursor is visible
    pub fn is_visible(&self) -> bool {
        self.visible
    }
    
    /// Set cursor visibility
    pub fn set_visible(&mut self, visible: bool) {
        self.visible = visible;
    }
}

impl Default for Cursor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_cursor_movement() {
        let mut cursor = Cursor::new();
        assert_eq!(cursor.position(), Position::new(0, 0));
        
        cursor.move_right(5);
        assert_eq!(cursor.position(), Position::new(0, 5));
        
        cursor.move_down(3);
        assert_eq!(cursor.position(), Position::new(3, 5));
        
        cursor.move_left(2);
        assert_eq!(cursor.position(), Position::new(3, 3));
        
        cursor.move_up(1);
        assert_eq!(cursor.position(), Position::new(2, 3));
    }
    
    #[test]
    fn test_cursor_save_restore() {
        let mut cursor = Cursor::new();
        
        cursor.set_position(Position::new(5, 10));
        cursor.save();
        
        cursor.set_position(Position::new(1, 1));
        assert_eq!(cursor.position(), Position::new(1, 1));
        
        cursor.restore();
        assert_eq!(cursor.position(), Position::new(5, 10));
    }
    
    #[test]
    fn test_cursor_saturating_movement() {
        let mut cursor = Cursor::new();
        
        // Move left from origin
        cursor.move_left(10);
        assert_eq!(cursor.position(), Position::new(0, 0));
        
        // Move up from origin
        cursor.move_up(10);
        assert_eq!(cursor.position(), Position::new(0, 0));
    }
}