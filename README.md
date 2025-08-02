# Phosphor Terminal

A modern, high-performance terminal emulator with authentic retro CRT aesthetics, built in Rust.

## Vision

Phosphor Terminal combines the functionality of modern terminals like iTerm2 with vintage computing aesthetics through GPU-accelerated shader effects. It's designed to be a professional-grade terminal emulator that can serve as a daily driver for developers while offering unique retro visual effects.

## Project Status

🚧 **Phase 1: Foundation (In Progress)** 🚧

Currently implementing the core terminal engine with basic PTY management and terminal state machine.

## Architecture

The project uses a multi-crate Rust workspace structure:

- **phosphor-core** - Terminal engine, PTY management, and session handling
- **phosphor-parser** - ANSI/VT escape sequence parsing and terminal protocols
- **phosphor-renderer** - GPU-accelerated rendering with WGPU and shader effects
- **phosphor-config** - Configuration management and theme system
- **phosphor-plugins** - WebAssembly-based plugin system
- **phosphor-gui** - GUI framework integration (Tauri or egui/eframe)

## Building from Source

### Prerequisites

- Rust 1.70 or later
- Git

### Build Instructions

```bash
# Clone the repository
git clone https://github.com/username/phosphor.git
cd phosphor

# Build the project
cargo build

# Run tests
cargo test

# Run the CLI test tool
cargo run --bin phosphor-cli
```

## Development

### Running Tests

```bash
# Run all tests
cargo test --workspace

# Run tests with output
cargo test --workspace -- --nocapture

# Run specific test
cargo test test_terminal_echo
```

### Code Quality

```bash
# Format code
cargo fmt

# Run linter
cargo clippy -- -D warnings

# Check without building
cargo check
```

## Phase 1 Features (Current)

- ✅ Multi-platform PTY support (Unix/macOS)
- ✅ Basic terminal state machine
- ✅ Screen buffer with scrollback
- ✅ Event-driven architecture
- ✅ Comprehensive error handling
- ✅ Structured logging with tracing

## Roadmap

### Phase 2: Terminal Protocol Support
- ANSI/VT100/VT220 escape sequences
- Color support (4-bit, 8-bit, 24-bit)
- Mouse reporting
- Alternative screen buffer

### Phase 3: Retro Visual Effects
- GPU-accelerated rendering with WGPU
- CRT shader effects (scanlines, phosphor glow, curvature)
- Multiple retro themes (Amber, Green, White phosphor)
- Authentic CRT simulation

### Phase 4: Modern Terminal Features
- Tabs and panes
- Search functionality
- URL/path detection
- Smart selection

### Phase 5: Configuration and Themes
- TOML-based configuration
- Live configuration reloading
- Theme engine
- Profile system

See [plan.md](plan.md) for the complete roadmap.

## Contributing

This project is in early development. Contributions are welcome! Please read the contributing guidelines before submitting PRs.

## License

This project is dual-licensed under MIT OR Apache-2.0.