use phosphor_common::types::Size;
use phosphor_core::Terminal;
use tokio::time::Duration;

#[tokio::test]
async fn test_pty_nonblocking_fix() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== Testing PTY Non-blocking Fix ===");
    
    // Create terminal
    let size = Size::new(80, 24);
    let terminal = Terminal::new(size)?;
    let cmd_sender = terminal.command_sender();
    let mut event_receiver = terminal.event_receiver();
    
    // Start terminal
    let terminal_handle = tokio::spawn(async move {
        terminal.run().await
    });
    
    // Collect outputs
    let event_handle = tokio::spawn(async move {
        let mut outputs = Vec::new();
        let mut closed = false;
        
        // Collect events for up to 5 seconds
        let deadline = tokio::time::Instant::now() + Duration::from_secs(5);
        while tokio::time::Instant::now() < deadline && !closed {
            tokio::select! {
                event = event_receiver.recv() => {
                    if let Ok(event) = event {
                        use phosphor_core::events::Event;
                        match event {
                            Event::OutputReady(data) => {
                                let text = String::from_utf8_lossy(&data).to_string();
                                println!("Output: {:?}", text);
                                outputs.push(text);
                            }
                            Event::Closed => {
                                println!("Terminal closed");
                                closed = true;
                            }
                            _ => {}
                        }
                    }
                }
                _ = tokio::time::sleep(Duration::from_millis(100)) => {
                    // Check periodically
                }
            }
        }
        
        outputs
    });
    
    // Give terminal time to start and show prompt
    tokio::time::sleep(Duration::from_millis(1000)).await;
    
    // Send commands
    println!("Sending newline...");
    cmd_sender.send(phosphor_core::events::Command::Write(vec![b'\n'])).await?;
    tokio::time::sleep(Duration::from_millis(500)).await;
    
    println!("Sending 'echo test'...");
    cmd_sender.send(phosphor_core::events::Command::Write(b"echo test\n".to_vec())).await?;
    tokio::time::sleep(Duration::from_millis(500)).await;
    
    println!("Sending 'echo hello'...");
    cmd_sender.send(phosphor_core::events::Command::Write(b"echo hello\n".to_vec())).await?;
    tokio::time::sleep(Duration::from_millis(500)).await;
    
    // Close terminal
    println!("Closing terminal...");
    cmd_sender.send(phosphor_core::events::Command::Close).await?;
    
    // Wait for results
    let _ = tokio::time::timeout(Duration::from_secs(2), terminal_handle).await;
    let outputs = tokio::time::timeout(Duration::from_secs(2), event_handle).await??;
    
    // Check results
    println!("\nReceived {} outputs", outputs.len());
    let combined = outputs.join("");
    
    // The fix is successful if:
    // 1. We received multiple outputs (not just initial prompt)
    // 2. The terminal continued responding after the first input
    // 3. We see our echo commands in the output
    
    if outputs.len() <= 2 {
        panic!("Terminal stopped responding after first input! Only got {} outputs", outputs.len());
    }
    
    if !combined.contains("test") || !combined.contains("hello") {
        panic!("Expected output not found. Got: {}", combined);
    }
    
    println!("\n✅ Test passed! Terminal continues working after input");
    println!("   - Received {} outputs", outputs.len());
    println!("   - Shell responded to multiple commands");
    
    Ok(())
}