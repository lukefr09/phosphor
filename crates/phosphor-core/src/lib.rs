pub mod ansi;
pub mod events;
pub mod pty;
pub mod session;
pub mod terminal;

use phosphor_common::{error::Result, types::Size, traits::{TerminalBackend, TerminalParser}};
use phosphor_parser::VteParser;
use tracing::{debug, info, error, instrument};

pub use events::EventBus;
pub use pty::PtyManager;
pub use terminal::TerminalState;

/// Main terminal structure that coordinates all components
pub struct Terminal {
    pty: PtyManager,
    state: TerminalState,
    parser: VteParser,
    event_bus: EventBus,
    size: Size,
}

impl Terminal {
    /// Create a new terminal with the specified size
    #[instrument]
    pub fn new(size: Size) -> Result<Self> {
        info!("Creating new Terminal with size: {:?}", size);
        let pty = PtyManager::spawn_shell(size)?;
        let state = TerminalState::new(size);
        let parser = VteParser::new();
        let event_bus = EventBus::new();
        
        info!("Terminal created successfully");
        Ok(Self { pty, state, parser, event_bus, size })
    }
    
    /// Get a command sender for external control
    pub fn command_sender(&self) -> tokio::sync::mpsc::Sender<events::Command> {
        self.event_bus.command_sender()
    }
    
    /// Get an event receiver for monitoring terminal events
    pub fn event_receiver(&self) -> tokio::sync::broadcast::Receiver<events::Event> {
        self.event_bus.event_receiver()
    }
    
    /// Run the terminal event loop
    #[instrument(skip(self))]
    pub async fn run(mut self) -> Result<()> {
        info!("Starting Terminal run loop");
        let mut buffer = vec![0u8; 4096];
        let event_tx = self.event_bus.event_sender();
        
        // Spawn command processor
        let mut command_rx = self.event_bus.take_command_receiver();
        let mut pty_writer = self.pty.clone();
        let cmd_processor = tokio::spawn(async move {
            debug!("Command processor started");
            while let Some(cmd) = command_rx.recv().await {
                use events::Command;
                match cmd {
                    Command::Write(data) => {
                        debug!("Processing write command: {} bytes", data.len());
                        if let Err(e) = pty_writer.write(&data).await {
                            error!("PTY write error: {}", e);
                            break;
                        }
                    }
                    Command::Resize(size) => {
                        debug!("Processing resize command: {:?}", size);
                        if let Err(e) = pty_writer.resize(size).await {
                            error!("PTY resize error: {}", e);
                        }
                    }
                    Command::Close => {
                        info!("Received close command");
                        break;
                    }
                }
            }
            debug!("Command processor exiting");
        });
        
        // Initial PTY alive check
        if !self.pty.is_alive().await {
            error!("PTY process is not alive before starting read loop!");
            return Err(phosphor_common::error::PhosphorError::Pty("PTY process died immediately".to_string()));
        }
        
        info!("Starting main read loop");
        let mut iteration = 0;
        
        // Send a minimal test input after a short delay to verify input works
        let test_sender = self.event_bus.command_sender();
        let test_size = self.size;
        tokio::spawn(async move {
            // First ensure the PTY has the right size
            tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
            info!("Sending resize command to ensure proper size");
            if let Err(e) = test_sender.send(events::Command::Resize(test_size)).await {
                error!("Failed to send resize: {}", e);
            }
            
            tokio::time::sleep(tokio::time::Duration::from_millis(300)).await;
            info!("Sending minimal test input: just newline");
            let test_cmd = b"\n";
            if let Err(e) = test_sender.send(events::Command::Write(test_cmd.to_vec())).await {
                error!("Failed to send test input: {}", e);
            }
            
            // Try another test after a bit more time
            tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
            info!("Sending second test: 'pwd' command");
            let test_cmd2 = b"pwd\n";
            if let Err(e) = test_sender.send(events::Command::Write(test_cmd2.to_vec())).await {
                error!("Failed to send second test: {}", e);
            }
        });
        
        // Main read loop
        loop {
            iteration += 1;
            debug!("Read loop iteration: {}", iteration);
            
            tokio::select! {
                // Read from PTY
                result = self.pty.read(&mut buffer) => {
                    match result {
                        Ok(0) => {
                            // With non-blocking I/O, 0 bytes doesn't necessarily mean EOF
                            // It could just mean no data is available right now
                            // We rely on the is_alive check to detect when the PTY actually closes
                            debug!("PTY read returned 0 bytes (no data available)");
                            // Don't break here - continue the loop
                        }
                        Ok(n) => {
                            info!("PTY read successful: {} bytes", n);
                            let data = &buffer[..n];
                            self.process_output(data)?;
                            
                            // Send event
                            let _ = event_tx.send(events::Event::OutputReady(data.to_vec()));
                        }
                        Err(e) => {
                            error!("PTY read error: {}", e);
                            return Err(e);
                        }
                    }
                }
                
                // Check if PTY is still alive
                _ = tokio::time::sleep(tokio::time::Duration::from_secs(1)) => {
                    debug!("Checking PTY alive status");
                    if !self.pty.is_alive().await {
                        info!("PTY process ended (detected in alive check)");
                        break;
                    }
                }
            }
        }
        
        info!("Exiting main read loop");
        
        // Clean up
        let _ = event_tx.send(events::Event::Closed);
        let _ = cmd_processor.await;
        
        info!("Terminal run loop completed");
        Ok(())
    }
    
    fn process_output(&mut self, data: &[u8]) -> Result<()> {
        // Parse the data and process events
        let events = self.parser.parse(data);
        for event in events {
            ansi::AnsiProcessor::process_event(&mut self.state, event);
        }
        
        // Send state changed event
        let _ = self.event_bus.event_sender().send(events::Event::StateChanged);
        
        Ok(())
    }
    
    /// Get the current terminal state
    pub fn state(&self) -> &TerminalState {
        &self.state
    }
    
    /// Get the current terminal size
    pub fn size(&self) -> Size {
        self.size
    }
}