# Fix PTY Blocking Reads

## Problem Diagnosis

After extensive testing, the root cause of the PTY "EOF" issue has been identified:

1. **Shells are working correctly** - They don't close their output streams
2. **PTY setup is correct** - portable-pty properly sets up setsid() and controlling terminal
3. **The issue is blocking reads** - When no data is available, read() blocks indefinitely

### Evidence

Testing showed:
- Non-shell programs (cat, echo, grep) work fine
- Shells (bash, zsh, sh) also work but appear to "hang" after input
- The shell processes remain alive even when reads block
- Multiple shells show prompts and execute commands successfully

### Root Cause

The Unix PTY reader from portable-pty performs **blocking reads**. When there's no data available (e.g., after a shell prompt is displayed), the read() call blocks forever. This is misinterpreted as an EOF in async contexts.

## Solution

The PTY file descriptor needs to be set to non-blocking mode so reads return immediately with EAGAIN/EWOULDBLOCK when no data is available.

### Implementation

```rust
// In AsyncPtyIo::new, after getting the reader
use std::os::unix::io::AsRawFd;
let fd = reader.as_raw_fd();
unsafe {
    let flags = libc::fcntl(fd, libc::F_GETFL, 0);
    libc::fcntl(fd, libc::F_SETFL, flags | libc::O_NONBLOCK);
}
```

### Alternative Approaches

1. **Use tokio-fd or async-std** - Properly integrate with async runtime
2. **Poll-based approach** - Use epoll/kqueue for readiness notifications
3. **Timeout-based reads** - Add timeouts to detect when no data is available

## Testing

The fix can be verified by:
1. Running the terminal and typing commands
2. Ensuring the shell continues to respond after multiple inputs
3. Checking that the reader properly handles EAGAIN errors