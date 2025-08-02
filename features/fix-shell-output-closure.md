# Fix Shell Output Closure After Input

## Problem
Shells were closing their output streams immediately after receiving input, suggesting they detected improper terminal setup after processing input.

## Solution
Implemented multiple approaches to fix the issue:

### 1. Bypass Shell Configuration Files
Added flags to skip shell initialization files that might interfere:
- **bash**: `--noprofile --norc` to skip ~/.bashrc and /etc/profile
- **zsh**: `--no-rcs` to skip all rc files
- **sh**: `-i` for interactive mode (where supported)

### 2. Minimal Environment Option
Added `--minimal-env` flag that uses `env -i` to start shells with minimal environment:
- Only essential variables: PATH, TERM, HOME, USER
- Bypasses all shell initialization
- Cleanest possible shell startup

### 3. Test with Minimal Input
Changed test from "echo PHOSPHOR_TEST" to just "\n" (newline):
- First test: Send empty newline after 500ms
- Second test: Send "pwd" command after 1500ms
- Helps identify if issue is input-specific

### 4. Enhanced Logging
- Log all shell spawn parameters
- Log PTY write operations with data content
- Added flush() after writes to ensure data delivery

## Usage

Test different configurations:
```bash
# Test with bypassed config files
cargo run --bin phosphor-cli -- --debug --shell /bin/bash

# Test with minimal environment
cargo run --bin phosphor-cli -- --debug --shell /bin/sh --minimal-env

# Run comprehensive tests
./test_shell_fixes.sh
```

## Implementation Details

The fixes are implemented in:
1. `crates/phosphor-core/src/pty/mod.rs` - Shell spawning logic
2. `crates/phosphor-cli/src/main.rs` - CLI arguments
3. `crates/phosphor-core/src/lib.rs` - Test input logic

## Debugging Output
The debug logs will show:
- Which shell flags are used
- Whether minimal environment is active
- When test inputs are sent
- PTY read/write operations
- Shell process status

This should help identify why shells close output after receiving input.