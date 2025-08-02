# Fix Shell Interactivity

## Problem
The shell was exiting immediately after displaying the prompt because it didn't recognize it was running in an interactive terminal session.

## Solution
Modified the PTY spawn process to ensure the shell runs in interactive mode:

### 1. Force Interactive Mode
- Added `-i` flag for bash and zsh shells to force interactive mode
- Special handling for POSIX sh which may not support `-i`

### 2. Environment Configuration
Set critical environment variables to ensure shell interactivity:
- `TERM=xterm-256color` - Terminal type
- `COLORTERM=truecolor` - Color support
- `PS1=\u@\h:\w\$` - Interactive prompt
- `SHELL` - Path to the shell itself
- `USER` - Current username
- `HOME` - Home directory
- `PATH` - Executable search path

### 3. Working Directory
- Set the current working directory for the shell process

### 4. Shell Initialization Check
- Added a 50ms delay after spawn to allow shell initialization
- Check if the process is still alive after spawn
- Return error if shell exits immediately with status code

### 5. Improved Error Handling
- Enhanced error messages to show exact exit status
- Better logging throughout the spawn process

## Implementation Details

The changes were made in `crates/phosphor-core/src/pty/mod.rs` in the `spawn_shell()` method:

```rust
// Force interactive mode based on shell type
if shell.contains("bash") || shell.contains("zsh") {
    cmd.arg("-i");
}

// Set up complete environment
cmd.env("TERM", "xterm-256color");
cmd.env("PS1", "\\u@\\h:\\w\\$ ");
// ... other environment variables

// Check shell is alive after spawn
match child.try_wait() {
    Ok(None) => info!("Shell process is running"),
    Ok(Some(status)) => return Err(...),
    Err(e) => error!("Error checking status: {}", e),
}
```

## Testing
To test the fix:
1. Run `cargo run --bin phosphor-cli -- --debug`
2. The shell should now stay interactive and accept commands
3. Check debug logs for successful shell initialization

## Common Shell Requirements
Different shells have different requirements for interactivity:
- **bash**: Requires `-i` flag or detection of tty
- **zsh**: Similar to bash, needs `-i` or tty detection
- **sh**: May not support `-i`, relies on environment detection
- **fish**: Auto-detects interactivity based on stdin/stdout

The current implementation handles the most common shells (bash, zsh, sh) appropriately.