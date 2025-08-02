use phosphor_common::types::{Cell, Position, Size};
use std::collections::VecDeque;

/// Screen buffer that holds the visible terminal content
pub struct ScreenBuffer {
    lines: Vec<Vec<Cell>>,
    size: Size,
}

impl ScreenBuffer {
    /// Create a new screen buffer with the given size
    pub fn new(size: Size) -> Self {
        let lines = (0..size.rows)
            .map(|_| vec![Cell::blank(); size.cols as usize])
            .collect();
        
        Self { lines, size }
    }
    
    /// Set a cell at the given position
    pub fn set_cell(&mut self, pos: Position, cell: Cell) {
        if pos.row < self.size.rows && pos.col < self.size.cols {
            self.lines[pos.row as usize][pos.col as usize] = cell;
        }
    }
    
    /// Get a cell at the given position
    pub fn get_cell(&self, pos: Position) -> Cell {
        if pos.row < self.size.rows && pos.col < self.size.cols {
            self.lines[pos.row as usize][pos.col as usize].clone()
        } else {
            Cell::blank()
        }
    }
    
    /// Get a reference to a specific line
    pub fn get_line(&self, row: u16) -> Option<&Vec<Cell>> {
        if row < self.size.rows {
            Some(&self.lines[row as usize])
        } else {
            None
        }
    }
    
    /// Remove the top line and return it
    pub fn remove_top_line(&mut self) -> Option<Vec<Cell>> {
        if !self.lines.is_empty() {
            Some(self.lines.remove(0))
        } else {
            None
        }
    }
    
    /// Add a blank line at the bottom
    pub fn add_blank_line(&mut self) {
        self.lines.push(vec![Cell::blank(); self.size.cols as usize]);
    }
    
    /// Clear the entire buffer
    pub fn clear(&mut self) {
        for line in &mut self.lines {
            for cell in line {
                *cell = Cell::blank();
            }
        }
    }
    
    /// Clear a line
    pub fn clear_line(&mut self, row: u16) {
        if row < self.size.rows {
            for cell in &mut self.lines[row as usize] {
                *cell = Cell::blank();
            }
        }
    }
    
    /// Resize the buffer
    pub fn resize(&mut self, new_size: Size) {
        // First resize columns for existing rows
        for line in &mut self.lines {
            if new_size.cols > self.size.cols {
                // Add blank cells
                line.extend((self.size.cols..new_size.cols).map(|_| Cell::blank()));
            } else if new_size.cols < self.size.cols {
                // Remove excess cells
                line.truncate(new_size.cols as usize);
            }
        }
        
        // Then resize rows
        if new_size.rows > self.size.rows {
            // Add new blank lines with the new column count
            for _ in self.size.rows..new_size.rows {
                self.lines.push(vec![Cell::blank(); new_size.cols as usize]);
            }
        } else if new_size.rows < self.size.rows {
            // Remove excess lines
            self.lines.truncate(new_size.rows as usize);
        }
        
        self.size = new_size;
    }
    
    /// Get the buffer size
    pub fn size(&self) -> Size {
        self.size
    }
    
    /// Get all lines as a slice
    pub fn lines(&self) -> &[Vec<Cell>] {
        &self.lines
    }
}

/// Scrollback buffer that holds historical terminal content
pub struct ScrollbackBuffer {
    lines: VecDeque<Vec<Cell>>,
    max_lines: usize,
}

impl ScrollbackBuffer {
    /// Create a new scrollback buffer with a maximum number of lines
    pub fn new(max_lines: usize) -> Self {
        Self {
            lines: VecDeque::with_capacity(max_lines.min(100_000)), // Cap capacity
            max_lines,
        }
    }
    
    /// Push a new line to the scrollback
    pub fn push(&mut self, line: Vec<Cell>) {
        if self.lines.len() >= self.max_lines {
            self.lines.pop_front();
        }
        self.lines.push_back(line);
    }
    
    /// Get the number of lines in scrollback
    pub fn len(&self) -> usize {
        self.lines.len()
    }
    
    /// Check if scrollback is empty
    pub fn is_empty(&self) -> bool {
        self.lines.is_empty()
    }
    
    /// Get a line from scrollback (0 is oldest)
    pub fn get_line(&self, index: usize) -> Option<&Vec<Cell>> {
        self.lines.get(index)
    }
    
    /// Clear the scrollback buffer
    pub fn clear(&mut self) {
        self.lines.clear();
    }
    
    /// Get all lines as a slice
    pub fn lines(&self) -> &VecDeque<Vec<Cell>> {
        &self.lines
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_screen_buffer_basic() {
        let mut buffer = ScreenBuffer::new(Size::new(10, 5));
        
        // Set and get cell
        let pos = Position::new(2, 3);
        buffer.set_cell(pos, Cell::new('A'));
        assert_eq!(buffer.get_cell(pos).ch, 'A');
        
        // Out of bounds
        let oob_pos = Position::new(10, 10);
        buffer.set_cell(oob_pos, Cell::new('B'));
        assert_eq!(buffer.get_cell(oob_pos).ch, ' ');
    }
    
    #[test]
    fn test_screen_buffer_resize() {
        let mut buffer = ScreenBuffer::new(Size::new(5, 3));
        
        // Set some content
        buffer.set_cell(Position::new(0, 0), Cell::new('A'));
        buffer.set_cell(Position::new(2, 4), Cell::new('B'));
        
        // Resize larger
        buffer.resize(Size::new(7, 5));
        assert_eq!(buffer.get_cell(Position::new(0, 0)).ch, 'A');
        assert_eq!(buffer.get_cell(Position::new(2, 4)).ch, 'B');
        assert_eq!(buffer.size(), Size::new(7, 5));
        
        // Resize smaller
        buffer.resize(Size::new(3, 2));
        assert_eq!(buffer.get_cell(Position::new(0, 0)).ch, 'A');
        assert_eq!(buffer.size(), Size::new(3, 2));
    }
    
    #[test]
    fn test_scrollback_buffer() {
        let mut scrollback = ScrollbackBuffer::new(3);
        
        // Push lines
        scrollback.push(vec![Cell::new('1')]);
        scrollback.push(vec![Cell::new('2')]);
        scrollback.push(vec![Cell::new('3')]);
        assert_eq!(scrollback.len(), 3);
        
        // Push beyond limit
        scrollback.push(vec![Cell::new('4')]);
        assert_eq!(scrollback.len(), 3);
        
        // Check that oldest was removed
        assert_eq!(scrollback.get_line(0).unwrap()[0].ch, '2');
        assert_eq!(scrollback.get_line(2).unwrap()[0].ch, '4');
    }
}