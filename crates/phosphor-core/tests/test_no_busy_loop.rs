use phosphor_common::types::Size;
use phosphor_core::Terminal;
use std::time::{Duration, Instant};
use tokio::time;

#[tokio::test]
async fn test_no_busy_loop() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== Testing No Busy Loop with Blocking I/O ===");
    
    // Create terminal
    let size = Size::new(80, 24);
    let terminal = Terminal::new(size)?;
    let cmd_sender = terminal.command_sender();
    let mut event_receiver = terminal.event_receiver();
    
    // Track timing
    let start = Instant::now();
    let mut read_count = 0;
    
    // Start terminal
    let terminal_handle = tokio::spawn(async move {
        terminal.run().await
    });
    
    // Count reads over 2 seconds
    let event_handle = tokio::spawn(async move {
        let deadline = time::Instant::now() + Duration::from_secs(2);
        
        while time::Instant::now() < deadline {
            tokio::select! {
                event = event_receiver.recv() => {
                    if let Ok(event) = event {
                        use phosphor_core::events::Event;
                        if let Event::OutputReady(_) = event {
                            read_count += 1;
                        }
                    }
                }
                _ = time::sleep(Duration::from_millis(10)) => {}
            }
        }
        
        read_count
    });
    
    // Wait for initial output
    time::sleep(Duration::from_millis(500)).await;
    
    // Send a command
    cmd_sender.send(phosphor_core::events::Command::Write(b"echo test\n".to_vec())).await?;
    
    // Wait for test to complete
    let final_count = time::timeout(Duration::from_secs(3), event_handle).await??;
    
    // Close terminal
    cmd_sender.send(phosphor_core::events::Command::Close).await?;
    let _ = time::timeout(Duration::from_secs(1), terminal_handle).await;
    
    let elapsed = start.elapsed();
    println!("Test ran for {:?}", elapsed);
    println!("Total reads: {}", final_count);
    
    // With blocking I/O, we should see:
    // - Initial prompt (1-2 reads)
    // - Echo response (1-2 reads)
    // - NOT hundreds of reads from a busy loop
    
    assert!(final_count < 20, "Too many reads ({}), likely a busy loop!", final_count);
    assert!(final_count >= 2, "Too few reads ({}), terminal not working!", final_count);
    
    println!("✅ No busy loop detected - proper blocking I/O!");
    
    Ok(())
}