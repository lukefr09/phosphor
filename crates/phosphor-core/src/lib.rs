pub mod events;
pub mod pty;
pub mod session;
pub mod terminal;

use phosphor_common::{error::Result, types::Size, traits::TerminalBackend};
use tracing::instrument;

pub use events::EventBus;
pub use pty::PtyManager;
pub use terminal::TerminalState;

/// Main terminal structure that coordinates all components
pub struct Terminal {
    pty: PtyManager,
    state: TerminalState,
    event_bus: EventBus,
    size: Size,
}

impl Terminal {
    /// Create a new terminal with the specified size
    #[instrument]
    pub fn new(size: Size) -> Result<Self> {
        let pty = PtyManager::spawn_shell(size)?;
        let state = TerminalState::new(size);
        let event_bus = EventBus::new();
        
        Ok(Self { pty, state, event_bus, size })
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
        let mut buffer = vec![0u8; 4096];
        let event_tx = self.event_bus.event_sender();
        
        // Spawn command processor
        let mut command_rx = self.event_bus.take_command_receiver();
        let mut pty_writer = self.pty.clone();
        let cmd_processor = tokio::spawn(async move {
            while let Some(cmd) = command_rx.recv().await {
                use events::Command;
                match cmd {
                    Command::Write(data) => {
                        if let Err(e) = pty_writer.write(&data).await {
                            tracing::error!("PTY write error: {}", e);
                            break;
                        }
                    }
                    Command::Resize(size) => {
                        if let Err(e) = pty_writer.resize(size).await {
                            tracing::error!("PTY resize error: {}", e);
                        }
                    }
                    Command::Close => {
                        tracing::info!("Received close command");
                        break;
                    }
                }
            }
        });
        
        // Main read loop
        loop {
            tokio::select! {
                // Read from PTY
                result = self.pty.read(&mut buffer) => {
                    match result {
                        Ok(0) => {
                            tracing::info!("PTY closed");
                            break;
                        }
                        Ok(n) => {
                            let data = &buffer[..n];
                            self.process_output(data)?;
                            
                            // Send event
                            let _ = event_tx.send(events::Event::OutputReady(data.to_vec()));
                        }
                        Err(e) => {
                            tracing::error!("PTY read error: {}", e);
                            return Err(e);
                        }
                    }
                }
                
                // Check if PTY is still alive
                _ = tokio::time::sleep(tokio::time::Duration::from_secs(1)) => {
                    if !self.pty.is_alive().await {
                        tracing::info!("PTY process ended");
                        break;
                    }
                }
            }
        }
        
        // Clean up
        let _ = event_tx.send(events::Event::Closed);
        let _ = cmd_processor.await;
        
        Ok(())
    }
    
    fn process_output(&mut self, data: &[u8]) -> Result<()> {
        // For Phase 1, just handle plain text
        let text = String::from_utf8_lossy(data);
        for ch in text.chars() {
            self.state.write_char(ch);
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