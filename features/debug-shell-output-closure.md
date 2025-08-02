# Debug Shell Output Closure

## Problem
The shell starts successfully but closes its output stream after ~1 second. The process stays alive but closes stdout/stderr, causing EOF. This suggests the shell can't read input properly.

## Debugging Approach

### 1. Test Shell Input
Added automatic test command that sends "echo PHOSPHOR_TEST\n" after 500ms to verify if the shell can receive and process input.

### 2. Enhanced PTY Write Logging
Added comprehensive logging to the AsyncPtyIo write path:
- Log all write attempts with data content
- Log successful writes with byte count
- Added flush() after writes to ensure data is sent
- Log any write errors

### 3. Shell Configuration Options
Added CLI flag `--shell` to test with different shells:
```bash
# Test with sh
cargo run --bin phosphor-cli -- --debug --shell /bin/sh

# Test with bash
cargo run --bin phosphor-cli -- --debug --shell /bin/bash

# Test with zsh (default)
cargo run --bin phosphor-cli -- --debug
```

### 4. Shell-Specific Flags
Enhanced shell startup flags:
- bash: `-i` for interactive mode
- zsh: `-i -l` for interactive login shell
- sh: `-i` (may not be supported by all sh implementations)

## Potential Issues to Investigate

1. **PTY Slave Configuration**: The slave PTY might need proper terminal settings
2. **File Descriptor Issues**: The shell might be detecting closed/invalid file descriptors
3. **Signal Handling**: The shell might be receiving unexpected signals
4. **Environment Detection**: The shell might not detect it's in an interactive terminal

## Next Steps

1. Check if the test command produces output
2. Monitor PTY write operations to see if data reaches the shell
3. Compare behavior between different shells
4. Consider using `stty` settings on the PTY slave side
5. Check if we need to handle SIGCHLD or other signals

## Testing Commands

```bash
# Test with debug output
cargo run --bin phosphor-cli -- --debug

# Test with sh
cargo run --bin phosphor-cli -- --debug --shell /bin/sh

# Test with bash  
cargo run --bin phosphor-cli -- --debug --shell /bin/bash

# Run with environment debugging
RUST_LOG=phosphor=trace cargo run --bin phosphor-cli -- --debug
```