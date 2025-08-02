# Fix PTY Configuration

## Problem
Shells were closing their output streams immediately after receiving ANY input, even with minimal environment. This indicated a fundamental PTY configuration issue where shells detected the PTY wasn't a "real" terminal.

## Root Cause Analysis
The issue was related to improper PTY session management:
1. The slave PTY handle wasn't being dropped after spawning the shell
2. The shell wasn't properly becoming the session leader
3. The controlling terminal wasn't explicitly set

## Solution

### 1. Drop Slave PTY Handle
**Critical fix**: After spawning the shell, we must drop the slave PTY handle to relinquish control to the child process.
```rust
let mut child = pair.slave.spawn_command(cmd)?;
drop(pair.slave);  // IMPORTANT: Relinquish slave to child
```

### 2. Explicitly Set Controlling Terminal
While this is the default in portable-pty, we now explicitly set it:
```rust
cmd.set_controlling_tty(true);
```

### 3. Ensure Proper Terminal Size
Added resize command early in the session to ensure the PTY has proper dimensions:
```rust
// Send resize to ensure PTY knows its size
test_sender.send(Command::Resize(size)).await
```

## Implementation Details

### Session Management
When a shell is spawned with `set_controlling_tty(true)`:
- The shell becomes the session leader (via setsid())
- The PTY becomes the controlling terminal
- The shell is placed in the foreground process group

### Why Dropping Slave Matters
- The slave file descriptor must be owned exclusively by the child
- Keeping it open in the parent interferes with terminal I/O
- This causes shells to detect improper terminal setup

### portable-pty Behavior
portable-pty handles most of the low-level details:
- Calls setsid() to create new session
- Sets the PTY as controlling terminal (TIOCSCTTY)
- Configures process groups correctly

## Testing
To verify the fix works:
```bash
# Test with debug output
cargo run --bin phosphor-cli -- --debug

# Test different shells
cargo run --bin phosphor-cli -- --debug --shell /bin/bash
cargo run --bin phosphor-cli -- --debug --shell /bin/sh
```

## Key Learnings
1. **Always drop the slave** - This is documented but easy to miss
2. **Controlling terminal is critical** - Without it, shells won't stay interactive
3. **Session management matters** - Shells need proper session/process group setup
4. **portable-pty does the heavy lifting** - But you must use it correctly

## References
- portable-pty documentation on CommandBuilder
- Linux PTY programming guides on session management
- setsid(2) and ioctl(2) man pages for TIOCSCTTY