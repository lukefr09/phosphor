# Phase 1: Foundation Implementation Plan

## Overview
This document outlines the detailed implementation steps for Phase 1 of the Phosphor Terminal project, focusing on building the core foundation with PTY management, terminal state, and event system.

## Step 1: Workspace Setup and Infrastructure

### 1.1 Create Rust Workspace Structure
```bash
# Root workspace Cargo.toml
phosphor/
├── Cargo.toml              # Workspace root
├── crates/
│   ├── phosphor-core/      # Terminal engine & PTY
│   ├── phosphor-parser/    # Minimal parser stub
│   ├── phosphor-common/    # Shared types and traits
│   └── phosphor-cli/       # CLI binary for testing
├── tests/                  # Integration tests
├── docs/                   # Documentation
└── .github/               # CI/CD workflows
```

### 1.2 Workspace Configuration
```toml
# Cargo.toml (root)
[workspace]
members = [
    "crates/phosphor-core",
    "crates/phosphor-parser", 
    "crates/phosphor-common",
    "crates/phosphor-cli",
]
resolver = "2"

[workspace.package]
version = "0.1.0"
edition = "2021"
authors = ["Phosphor Terminal Contributors"]
license = "MIT OR Apache-2.0"
repository = "https://github.com/username/phosphor"

[workspace.dependencies]
# Async runtime
tokio = { version = "1.35", features = ["full"] }
tokio-stream = "0.1"

# PTY handling
portable-pty = "0.8"

# Error handling
thiserror = "1.0"
anyhow = "1.0"

# Logging
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }

# Serialization
serde = { version = "1.0", features = ["derive"] }

# Testing
proptest = "1.4"
tempfile = "3.8"
```

### 1.3 Development Environment Setup
```toml
# rust-toolchain.toml
[toolchain]
channel = "stable"
components = ["rustfmt", "clippy"]
```

```toml
# .rustfmt.toml
edition = "2021"
use_field_init_shorthand = true
use_try_shorthand = true
```

```toml
# .clippy.toml
warn-by-default = true
```

### 1.4 CI/CD Setup
```yaml
# .github/workflows/ci.yml
name: CI

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

env:
  CARGO_TERM_COLOR: always

jobs:
  test:
    strategy:
      matrix:
        os: [ubuntu-latest, macos-latest]
    runs-on: ${{ matrix.os }}
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
      - run: cargo build --workspace
      - run: cargo test --workspace
      - run: cargo clippy --workspace -- -D warnings
      - run: cargo fmt --all -- --check
```

## Step 2: phosphor-common - Shared Types and Traits

### 2.1 Core Types
```rust
// crates/phosphor-common/src/lib.rs
pub mod types;
pub mod traits;
pub mod error;

// crates/phosphor-common/src/types.rs
use serde::{Deserialize, Serialize};

/// Terminal dimensions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Size {
    pub rows: u16,
    pub cols: u16,
}

/// Cursor position (0-indexed)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Position {
    pub row: u16,
    pub col: u16,
}

/// Character cell in the terminal
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cell {
    pub ch: char,
    pub attrs: CellAttributes,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct CellAttributes {
    pub fg_color: Option<Color>,
    pub bg_color: Option<Color>,
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Color {
    Rgb(u8, u8, u8),
    Indexed(u8),
}
```

### 2.2 Core Traits
```rust
// crates/phosphor-common/src/traits.rs
use crate::types::{Size, Position};
use std::future::Future;

/// Trait for terminal frontends (GUI frameworks)
pub trait TerminalFrontend: Send + Sync {
    /// Update the display with new terminal state
    fn update(&mut self, state: &TerminalState) -> Result<(), Box<dyn std::error::Error>>;
    
    /// Handle resize events
    fn resize(&mut self, size: Size) -> Result<(), Box<dyn std::error::Error>>;
}

/// Trait for terminal backends (PTY, parser, etc)
pub trait TerminalBackend: Send + Sync {
    /// Write data to the terminal
    fn write(&mut self, data: &[u8]) -> impl Future<Output = Result<usize, std::io::Error>>;
    
    /// Read data from the terminal
    fn read(&mut self, buf: &mut [u8]) -> impl Future<Output = Result<usize, std::io::Error>>;
    
    /// Resize the terminal
    fn resize(&mut self, size: Size) -> impl Future<Output = Result<(), std::io::Error>>;
}
```

### 2.3 Error Types
```rust
// crates/phosphor-common/src/error.rs
use thiserror::Error;

#[derive(Error, Debug)]
pub enum PhosphorError {
    #[error("PTY error: {0}")]
    Pty(String),
    
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Parse error: {0}")]
    Parse(String),
    
    #[error("Terminal state error: {0}")]
    State(String),
    
    #[error("Event system error: {0}")]
    Event(String),
}

pub type Result<T> = std::result::Result<T, PhosphorError>;
```

## Step 3: phosphor-core Implementation

### 3.1 Module Structure
```
phosphor-core/
├── src/
│   ├── lib.rs
│   ├── pty/
│   │   ├── mod.rs
│   │   ├── unix.rs      # Unix/macOS implementation
│   │   └── windows.rs   # Windows stubs
│   ├── terminal/
│   │   ├── mod.rs
│   │   ├── state.rs     # Terminal state machine
│   │   ├── buffer.rs    # Screen and scrollback buffers
│   │   └── cursor.rs    # Cursor management
│   ├── events/
│   │   ├── mod.rs
│   │   ├── bus.rs       # Event bus implementation
│   │   └── types.rs     # Event types
│   └── session/
│       ├── mod.rs
│       └── manager.rs   # Basic session management
└── tests/
```

### 3.2 PTY Management
```rust
// crates/phosphor-core/src/pty/mod.rs
use phosphor_common::{traits::TerminalBackend, types::Size, error::Result};
use portable_pty::{native_pty_system, CommandBuilder, PtySize};
use tokio::io::{AsyncRead, AsyncWrite};
use tracing::{debug, error, instrument};

pub struct PtyManager {
    pty: Box<dyn portable_pty::MasterPty + Send>,
    reader: Box<dyn AsyncRead + Send + Unpin>,
    writer: Box<dyn AsyncWrite + Send + Unpin>,
}

impl PtyManager {
    #[instrument]
    pub fn spawn_shell(size: Size) -> Result<Self> {
        let pty_system = native_pty_system();
        let pty_size = PtySize {
            rows: size.rows,
            cols: size.cols,
            pixel_width: 0,
            pixel_height: 0,
        };
        
        let pair = pty_system.openpty(pty_size)
            .map_err(|e| PhosphorError::Pty(e.to_string()))?;
        
        let cmd = CommandBuilder::new(std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string()));
        let _child = pair.slave.spawn_command(cmd)
            .map_err(|e| PhosphorError::Pty(e.to_string()))?;
            
        // Convert to async I/O
        let reader = pair.master.try_clone_reader()
            .map_err(|e| PhosphorError::Pty(e.to_string()))?;
        let writer = pair.master.try_clone_writer()
            .map_err(|e| PhosphorError::Pty(e.to_string()))?;
            
        Ok(Self {
            pty: pair.master,
            reader: Box::new(tokio_util::compat::FuturesAsyncReadCompatExt::compat(reader)),
            writer: Box::new(tokio_util::compat::FuturesAsyncWriteCompatExt::compat_write(writer)),
        })
    }
}

#[async_trait::async_trait]
impl TerminalBackend for PtyManager {
    async fn write(&mut self, data: &[u8]) -> Result<usize> {
        use tokio::io::AsyncWriteExt;
        self.writer.write(data).await.map_err(Into::into)
    }
    
    async fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        use tokio::io::AsyncReadExt;
        self.reader.read(buf).await.map_err(Into::into)
    }
    
    async fn resize(&mut self, size: Size) -> Result<()> {
        let pty_size = PtySize {
            rows: size.rows,
            cols: size.cols,
            pixel_width: 0,
            pixel_height: 0,
        };
        self.pty.resize(pty_size)
            .map_err(|e| PhosphorError::Pty(e.to_string()))?;
        Ok(())
    }
}
```

### 3.3 Terminal State Machine
```rust
// crates/phosphor-core/src/terminal/state.rs
use phosphor_common::types::{Size, Position, Cell};
use std::collections::VecDeque;
use tracing::instrument;

pub struct TerminalState {
    size: Size,
    cursor: Position,
    screen_buffer: ScreenBuffer,
    scrollback_buffer: ScrollbackBuffer,
}

impl TerminalState {
    pub fn new(size: Size) -> Self {
        Self {
            size,
            cursor: Position::default(),
            screen_buffer: ScreenBuffer::new(size),
            scrollback_buffer: ScrollbackBuffer::new(10_000), // 10k lines
        }
    }
    
    #[instrument(skip(self))]
    pub fn write_char(&mut self, ch: char) {
        match ch {
            '\n' => self.new_line(),
            '\r' => self.carriage_return(),
            '\t' => self.tab(),
            _ => {
                self.screen_buffer.set_cell(self.cursor, Cell::from_char(ch));
                self.advance_cursor();
            }
        }
    }
    
    fn new_line(&mut self) {
        self.cursor.row += 1;
        if self.cursor.row >= self.size.rows {
            self.scroll_up();
            self.cursor.row = self.size.rows - 1;
        }
    }
    
    fn carriage_return(&mut self) {
        self.cursor.col = 0;
    }
    
    fn tab(&mut self) {
        // Move to next tab stop (every 8 columns)
        self.cursor.col = ((self.cursor.col / 8) + 1) * 8;
        if self.cursor.col >= self.size.cols {
            self.cursor.col = self.size.cols - 1;
        }
    }
    
    fn advance_cursor(&mut self) {
        self.cursor.col += 1;
        if self.cursor.col >= self.size.cols {
            self.cursor.col = 0;
            self.new_line();
        }
    }
    
    fn scroll_up(&mut self) {
        // Move top line to scrollback
        if let Some(line) = self.screen_buffer.remove_top_line() {
            self.scrollback_buffer.push(line);
        }
        self.screen_buffer.add_blank_line();
    }
}

pub struct ScreenBuffer {
    lines: Vec<Vec<Cell>>,
    size: Size,
}

impl ScreenBuffer {
    fn new(size: Size) -> Self {
        let lines = (0..size.rows)
            .map(|_| vec![Cell::default(); size.cols as usize])
            .collect();
        Self { lines, size }
    }
    
    fn set_cell(&mut self, pos: Position, cell: Cell) {
        if pos.row < self.size.rows && pos.col < self.size.cols {
            self.lines[pos.row as usize][pos.col as usize] = cell;
        }
    }
}

pub struct ScrollbackBuffer {
    lines: VecDeque<Vec<Cell>>,
    max_lines: usize,
}

impl ScrollbackBuffer {
    fn new(max_lines: usize) -> Self {
        Self {
            lines: VecDeque::with_capacity(max_lines),
            max_lines,
        }
    }
    
    fn push(&mut self, line: Vec<Cell>) {
        if self.lines.len() >= self.max_lines {
            self.lines.pop_front();
        }
        self.lines.push_back(line);
    }
}
```

### 3.4 Event System
```rust
// crates/phosphor-core/src/events/bus.rs
use tokio::sync::{mpsc, broadcast};
use phosphor_common::error::Result;
use tracing::{debug, instrument};

#[derive(Debug, Clone)]
pub enum Command {
    Write(Vec<u8>),
    Resize(Size),
    Close,
}

#[derive(Debug, Clone)]
pub enum Event {
    OutputReady(Vec<u8>),
    StateChanged,
    Resized(Size),
    Closed,
}

pub struct EventBus {
    command_tx: mpsc::Sender<Command>,
    command_rx: mpsc::Receiver<Command>,
    event_tx: broadcast::Sender<Event>,
}

impl EventBus {
    pub fn new() -> Self {
        let (command_tx, command_rx) = mpsc::channel(100);
        let (event_tx, _) = broadcast::channel(100);
        
        Self {
            command_tx,
            command_rx,
            event_tx,
        }
    }
    
    pub fn command_sender(&self) -> mpsc::Sender<Command> {
        self.command_tx.clone()
    }
    
    pub fn event_receiver(&self) -> broadcast::Receiver<Event> {
        self.event_tx.subscribe()
    }
    
    #[instrument(skip(self))]
    pub async fn process_commands(&mut self, backend: &mut impl TerminalBackend) -> Result<()> {
        while let Some(cmd) = self.command_rx.recv().await {
            match cmd {
                Command::Write(data) => {
                    backend.write(&data).await?;
                }
                Command::Resize(size) => {
                    backend.resize(size).await?;
                    self.event_tx.send(Event::Resized(size)).ok();
                }
                Command::Close => {
                    self.event_tx.send(Event::Closed).ok();
                    break;
                }
            }
        }
        Ok(())
    }
}
```

## Step 4: Integration and Event Loop

### 4.1 Main Event Loop
```rust
// crates/phosphor-core/src/lib.rs
use phosphor_common::{types::Size, error::Result};
use tracing::instrument;

pub struct Terminal {
    pty: PtyManager,
    state: TerminalState,
    event_bus: EventBus,
}

impl Terminal {
    pub fn new(size: Size) -> Result<Self> {
        let pty = PtyManager::spawn_shell(size)?;
        let state = TerminalState::new(size);
        let event_bus = EventBus::new();
        
        Ok(Self { pty, state, event_bus })
    }
    
    #[instrument(skip(self))]
    pub async fn run(mut self) -> Result<()> {
        let mut pty_reader = self.pty;
        let mut buffer = vec![0u8; 4096];
        
        // Spawn command processor
        let mut event_bus = self.event_bus;
        let cmd_processor = tokio::spawn(async move {
            event_bus.process_commands(&mut pty_reader).await
        });
        
        // Main read loop
        loop {
            tokio::select! {
                // Read from PTY
                result = pty_reader.read(&mut buffer) => {
                    match result {
                        Ok(0) => break, // EOF
                        Ok(n) => {
                            let data = &buffer[..n];
                            self.process_output(data)?;
                        }
                        Err(e) => return Err(e.into()),
                    }
                }
                
                // Handle other async events here
            }
        }
        
        cmd_processor.await??;
        Ok(())
    }
    
    fn process_output(&mut self, data: &[u8]) -> Result<()> {
        // For Phase 1, just handle plain text
        let text = String::from_utf8_lossy(data);
        for ch in text.chars() {
            self.state.write_char(ch);
        }
        Ok(())
    }
}
```

## Step 5: phosphor-cli Test Binary

### 5.1 CLI for Testing
```rust
// crates/phosphor-cli/src/main.rs
use anyhow::Result;
use phosphor_core::Terminal;
use phosphor_common::types::Size;
use tracing_subscriber;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter("phosphor=debug")
        .init();
    
    // Get terminal size
    let (cols, rows) = term_size::dimensions()
        .unwrap_or((80, 24));
    
    let size = Size {
        rows: rows as u16,
        cols: cols as u16,
    };
    
    // Create and run terminal
    let terminal = Terminal::new(size)?;
    terminal.run().await?;
    
    Ok(())
}
```

## Step 6: Testing Infrastructure

### 6.1 Unit Tests
```rust
// crates/phosphor-core/src/terminal/state.rs (tests module)
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_write_char() {
        let mut state = TerminalState::new(Size { rows: 24, cols: 80 });
        state.write_char('A');
        assert_eq!(state.cursor, Position { row: 0, col: 1 });
    }
    
    #[test]
    fn test_newline() {
        let mut state = TerminalState::new(Size { rows: 24, cols: 80 });
        state.write_char('\n');
        assert_eq!(state.cursor, Position { row: 1, col: 0 });
    }
    
    #[test]
    fn test_line_wrap() {
        let mut state = TerminalState::new(Size { rows: 24, cols: 3 });
        state.write_char('A');
        state.write_char('B');
        state.write_char('C');
        state.write_char('D');
        assert_eq!(state.cursor, Position { row: 1, col: 1 });
    }
}
```

### 6.2 Integration Tests
```rust
// tests/pty_integration.rs
use phosphor_core::Terminal;
use phosphor_common::types::Size;

#[tokio::test]
async fn test_echo_command() {
    let size = Size { rows: 24, cols: 80 };
    let mut terminal = Terminal::new(size).unwrap();
    
    // Send echo command
    terminal.write(b"echo hello\n").await.unwrap();
    
    // Read output
    let mut output = vec![0u8; 1024];
    let n = terminal.read(&mut output).await.unwrap();
    let text = String::from_utf8_lossy(&output[..n]);
    
    assert!(text.contains("hello"));
}
```

## Next Steps

1. **Create the workspace structure** with all the crates
2. **Implement phosphor-common** with shared types and traits
3. **Build phosphor-core** with PTY management and terminal state
4. **Add the event system** to connect components
5. **Create phosphor-cli** for testing
6. **Write comprehensive tests** for all components
7. **Set up CI/CD** with GitHub Actions
8. **Document the architecture** and API

## Timeline

- **Week 1**: Workspace setup and phosphor-common
- **Week 2**: PTY implementation and basic I/O
- **Week 3**: Terminal state machine and buffers
- **Week 4**: Event system and integration
- **Week 5**: Testing, documentation, and polish

This implementation provides a solid foundation that:
- Is GUI-agnostic with trait-based design
- Focuses on Unix/macOS with Windows stubs
- Uses Tokio for async operations
- Has proper error handling with thiserror/anyhow
- Includes tracing from the start
- Provides a testable architecture