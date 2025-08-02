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

### Next Steps (Phase 2):
- Integrate `vte` crate for ANSI escape sequence parsing
- Implement full VT100/VT220 compatibility
- Add color support (4-bit, 8-bit, 24-bit)
- Support alternate screen buffer
- Mouse event handling 

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