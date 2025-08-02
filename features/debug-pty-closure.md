# Debug PTY Closure Feature

## Overview
Added comprehensive debug logging throughout the PTY management and terminal event loop to diagnose why the PTY closes immediately after showing the shell prompt.

## Changes Made

### 1. Enhanced PTY Manager Logging (`crates/phosphor-core/src/pty/mod.rs`)
- Added info/error logging to `spawn_shell()` method
- Added debug logging to `read()` and `write()` methods with byte counts
- Enhanced `is_alive()` to log process exit status details

### 2. Enhanced Unix PTY I/O Logging (`crates/phosphor-core/src/pty/unix.rs`)
- Added logging to `AsyncPtyIo::new()` for initialization tracking
- Added error logging for read/write failures
- Added debug logging for would-block scenarios

### 3. Enhanced Terminal Event Loop Logging (`crates/phosphor-core/src/lib.rs`)
- Added initial PTY alive check before starting read loop
- Added iteration counter for read loop debugging
- Added comprehensive logging for all state transitions
- Added error handling for immediate PTY death

### 4. Enhanced CLI Demo Logging (`crates/phosphor-cli/src/main.rs`)
- Added event handler lifecycle logging
- Added debug logging for keyboard input
- Added error logging for stdout write failures

## Usage

To see the debug output, run the CLI demo with debug logging enabled:

```bash
cargo run --bin phosphor-cli -- --debug
```

Or set the environment variable:
```bash
RUST_LOG=phosphor=debug cargo run --bin phosphor-cli
```

## Debug Output Interpretation

The enhanced logging will show:
1. **PTY Initialization**: Whether the PTY and shell process start successfully
2. **Shell Exit Status**: The exact exit code if the shell terminates
3. **I/O Operations**: All read/write operations with byte counts
4. **Event Flow**: The sequence of events in the terminal loop
5. **Error Details**: Specific error messages for any failures

## Common Issues to Look For

1. **Shell Not Found**: Check if the SHELL environment variable points to a valid executable
2. **Permission Denied**: Ensure the user has permission to spawn the shell
3. **Immediate Exit**: Shell might be exiting due to profile/rc file errors
4. **I/O Errors**: File descriptor issues or PTY configuration problems

## Next Steps

After running with debug logging, examine the output for:
- Whether the shell process starts successfully
- The exit status if it terminates
- Any I/O errors during read/write operations
- The timing of when the PTY closes relative to other events