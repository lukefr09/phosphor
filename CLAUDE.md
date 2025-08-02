# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is the Phosphor Terminal project - a modern, high-performance terminal emulator with authentic retro CRT aesthetics, built in Rust. The project aims to combine the functionality of modern terminals like iTerm2 with vintage computing aesthetics through GPU-accelerated shader effects. Read the plan file before you do anything else. Always create a detailed plan before executing anything. Ask clarifying questions to flesh out more details for all user requests, then present an action plan for approval. Repeat and refine as many times as necessary. When you are finished implementing a phase, please update the Claude.md file with context. If no changes are needed dont change it.

## IMPORTANT

Create a .md file for each feature you add. Make sure it is well documented. Then put the file in the /features folder.

## Phase 1 Status: COMPLETED ✅

Phase 1 (Foundation) has been successfully implemented with the following components:

### Implemented Features:
1. **Multi-crate Rust workspace** - Clean architecture with separation of concerns
2. **PTY Management** - Full Unix/macOS support, Windows stubs
3. **Terminal State Machine** - Screen buffer, scrollback (10k lines), cursor management
4. **Event System** - Tokio-based async event bus with commands and events
5. **Error Handling** - `thiserror` for libraries, `anyhow` for application
6. **Logging** - Structured logging with `tracing`
7. **Testing** - Unit tests in all modules, integration tests for PTY
8. **CI/CD** - GitHub Actions workflow for multi-platform testing
9. **Documentation** - README, phase documentation, inline rustdoc

### Key Architecture Decisions:
- **Headless Core**: GUI-agnostic design with trait-based abstractions
- **Async-First**: All I/O operations use Tokio async/await
- **Platform Abstraction**: Clean separation with `#[cfg]` attributes
- **Event-Driven**: Decoupled components communicate via event bus

## Phase 2 Status: COMPLETED ✅

Phase 2 (Advanced Terminal Protocol Support) has been successfully implemented with:

### Implemented Features:
1. **VTE Integration** - Industry-standard escape sequence parser
2. **Full Color Support** - 4-bit ANSI, 8-bit indexed, 24-bit RGB
3. **Text Attributes** - Bold, italic, underline, reverse, dim, strikethrough
4. **ANSI Sequences** - Cursor control, screen manipulation, SGR
5. **Alternate Screen** - Full support for vim, less, htop
6. **Terminal Modes** - Line wrap, bracketed paste, application modes
7. **Enhanced Cell Type** - Colors, attributes, hyperlink support
8. **ANSI Processor** - Converts parsed events to state changes

### Technical Highlights:
- Bitflags for efficient attribute storage
- 256-color palette with proper defaults
- Cursor styles and visibility control
- Tab stop management
- Save/restore cursor position

### Recent Fixes:
1. **Shell Interactivity** - Fixed shell exiting immediately by:
   - Adding `-i` flag for bash/zsh interactive mode
   - Setting proper environment (PS1, USER, HOME, PATH)
   - Checking shell status after spawn
2. **Zero-size Terminal Handling** - Fixed arithmetic overflow by:
   - Adding size validation in terminal state operations
   - Using saturating arithmetic for boundary calculations
   - Validating terminal size on startup
3. **PTY Async Architecture** - Fixed shells appearing to close output after input by:
   - Using Arc<Mutex<>> pattern for safe reader/writer sharing across spawn_blocking
   - Keeping blocking I/O (removed non-blocking mode that caused busy loops)
   - Fixed the mem::replace pattern that was replacing reader with io::empty()
   - Shell processes now continue running and responding to multiple commands

### Next Steps (Phase 3):
- GPU-accelerated rendering with WGPU
- CRT shader effects and visual enhancements
- Font rendering with ligatures
- Performance optimizations 

## Architecture

The project uses a multi-crate Rust workspace structure:

- `phosphor-core/` - Terminal engine, PTY management, and session handling
- `phosphor-parser/` - ANSI/VT escape sequence parsing and terminal protocols
- `phosphor-renderer/` - GPU-accelerated rendering with WGPU and shader effects
- `phosphor-config/` - Configuration management and theme system
- `phosphor-plugins/` - WebAssembly-based plugin system
- `phosphor-gui/` - GUI framework integration (Tauri or egui/eframe)

## Development Commands

Since this is a Rust project in planning phase, once implementation begins:

```bash
# Build the project
cargo build

# Run in development mode
cargo run

# Run tests
cargo test

# Run a specific crate's tests
cargo test -p phosphor-core

# Check code without building
cargo check

# Format code
cargo fmt

# Run linter
cargo clippy

# Build release version
cargo build --release

# Run benchmarks
cargo bench

# Generate documentation
cargo doc --open
```

## Key Technical Decisions

1. **Rendering**: Uses WGPU for GPU-accelerated rendering with custom GLSL shaders for CRT effects
2. **Terminal Engine**: Built on `portable-pty` for cross-platform PTY support and `vte` for parsing
3. **Async Runtime**: Uses Tokio for async I/O and concurrency
4. **Plugin System**: WebAssembly-based using Wasmtime for security and portability
5. **Configuration**: TOML-based hierarchical configuration with live reloading
6. **GUI Framework**: Choice between Tauri (web technologies) or egui/eframe (immediate mode)

## Implementation Phases

The project is structured in 10 phases:
1. Foundation (Core Engine) - PTY management and terminal state
2. Terminal Protocol Support - ANSI/VT compatibility
3. Retro Visual Effects - GPU rendering and CRT simulation
4. Modern Terminal Features - Tabs, panes, search
5. Configuration and Themes - TOML config and retro themes
6. Plugin System - WASM-based extensibility
7. Performance Optimization - Memory and rendering optimizations
8. Cross-Platform Support - macOS, Windows, Linux integration
9. Advanced Features - AI integration, remote capabilities
10. Testing and QA - Comprehensive test suite

## Performance Targets

- Startup time: < 50ms
- Memory: < 100MB baseline
- Latency: < 16ms (60 FPS)
- Throughput: 1M+ chars/second

## Testing Strategy

- Unit tests for each crate
- Integration tests for PTY communication
- Visual regression tests for rendering
- Performance benchmarks
- Fuzz testing for escape sequence parsing