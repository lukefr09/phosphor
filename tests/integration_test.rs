use phosphor_common::types::Size;
use phosphor_core::{events::Command, Terminal};
use std::time::Duration;
use tokio::time::timeout;

#[tokio::test]
async fn test_terminal_echo() {
    // Create a small terminal
    let size = Size::new(80, 24);
    let terminal = Terminal::new(size).expect("Failed to create terminal");
    
    let cmd_sender = terminal.command_sender();
    let mut event_receiver = terminal.event_receiver();
    
    // Spawn terminal
    let terminal_handle = tokio::spawn(async move {
        terminal.run().await
    });
    
    // Send echo command
    cmd_sender
        .send(Command::Write(b"echo hello\n".to_vec()))
        .await
        .expect("Failed to send command");
    
    // Wait for output
    let mut output = Vec::new();
    let start = tokio::time::Instant::now();
    
    while start.elapsed() < Duration::from_secs(2) {
        match timeout(Duration::from_millis(100), event_receiver.recv()).await {
            Ok(Ok(event)) => {
                use phosphor_core::events::Event;
                if let Event::OutputReady(data) = event {
                    output.extend_from_slice(&data);
                    
                    // Check if we've received the expected output
                    let output_str = String::from_utf8_lossy(&output);
                    if output_str.contains("hello") {
                        break;
                    }
                }
            }
            _ => continue,
        }
    }
    
    // Send close command
    cmd_sender.send(Command::Close).await.ok();
    
    // Wait for terminal to close
    timeout(Duration::from_secs(5), terminal_handle)
        .await
        .expect("Terminal didn't close in time")
        .expect("Terminal task panicked")
        .expect("Terminal returned error");
    
    // Verify output
    let output_str = String::from_utf8_lossy(&output);
    assert!(output_str.contains("hello"), "Output doesn't contain 'hello': {}", output_str);
}

#[tokio::test]
async fn test_terminal_resize() {
    let initial_size = Size::new(80, 24);
    let terminal = Terminal::new(initial_size).expect("Failed to create terminal");
    
    let cmd_sender = terminal.command_sender();
    let mut event_receiver = terminal.event_receiver();
    
    // Spawn terminal
    let terminal_handle = tokio::spawn(async move {
        terminal.run().await
    });
    
    // Send resize command
    let new_size = Size::new(100, 30);
    cmd_sender
        .send(Command::Resize(new_size))
        .await
        .expect("Failed to send resize");
    
    // Wait for resize event
    let start = tokio::time::Instant::now();
    let mut resized = false;
    
    while start.elapsed() < Duration::from_secs(2) {
        match timeout(Duration::from_millis(100), event_receiver.recv()).await {
            Ok(Ok(event)) => {
                use phosphor_core::events::Event;
                if let Event::Resized(size) = event {
                    assert_eq!(size, new_size);
                    resized = true;
                    break;
                }
            }
            _ => continue,
        }
    }
    
    assert!(resized, "Didn't receive resize event");
    
    // Cleanup
    cmd_sender.send(Command::Close).await.ok();
    timeout(Duration::from_secs(5), terminal_handle).await.ok();
}

#[tokio::test]
async fn test_terminal_state() {
    let size = Size::new(80, 24);
    let terminal = Terminal::new(size).expect("Failed to create terminal");
    
    // Check initial state
    assert_eq!(terminal.size(), size);
    assert_eq!(terminal.state().size(), size);
    assert_eq!(terminal.state().cursor_position().row, 0);
    assert_eq!(terminal.state().cursor_position().col, 0);
}

#[cfg(unix)]
#[tokio::test]
async fn test_shell_spawn() {
    use phosphor_core::pty::PtyManager;
    use phosphor_common::traits::TerminalBackend;
    
    let size = Size::new(80, 24);
    let mut pty = PtyManager::spawn_shell(size).expect("Failed to spawn shell");
    
    // Check that PTY is alive
    assert!(pty.is_alive().await);
    
    // Write a simple command
    let written = pty.write(b"exit\n").await.expect("Failed to write");
    assert_eq!(written, 5);
    
    // Wait a bit for shell to exit
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    // Check that PTY is no longer alive
    assert!(!pty.is_alive().await);
}