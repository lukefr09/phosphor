# Fix PTY Async Architecture

## Problem Analysis

The current implementation has multiple architectural flaws:

1. **Non-blocking I/O + spawn_blocking**: This creates a busy loop as non-blocking reads return immediately with EAGAIN
2. **mem::replace with empty()**: If spawn_blocking fails/cancels, the reader is lost
3. **1ms delay band-aid**: Doesn't solve the underlying architectural mismatch

## Root Cause

The original blocking I/O actually works fine. The issue was in the async wrapper:
- Using `std::mem::replace` with `io::empty()` means subsequent reads after any error read from empty
- The PTY was never closing - we were just reading from the wrong source

## Solution: Proper Async Wrapper with Blocking I/O

Keep blocking I/O (remove non-blocking mode) and implement a proper async wrapper:

### Option 1: Arc<Mutex<>> Pattern (Recommended)
```rust
pub struct AsyncPtyIo {
    reader: Arc<Mutex<Box<dyn Read + Send>>>,
    writer: Arc<Mutex<Box<dyn Write + Send>>>,
}
```

### Option 2: Channel-based Reader
Create a dedicated thread that reads from the PTY and sends data via channels.

### Option 3: Use tokio-pty or pty-process
Switch to a crate that properly implements AsyncRead/AsyncWrite.

## Implementation Plan

1. Revert non-blocking mode changes
2. Implement Arc<Mutex<>> wrapper to safely share reader across spawn_blocking calls
3. Remove the 1ms delay hack
4. Test thoroughly to ensure no busy loops or false EOFs