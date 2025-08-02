use tokio::sync::{mpsc, broadcast};
use tracing::{debug, instrument};

use super::types::{Command, Event};

/// Event bus for coordinating between terminal components
pub struct EventBus {
    command_tx: mpsc::Sender<Command>,
    command_rx: Option<mpsc::Receiver<Command>>,
    event_tx: broadcast::Sender<Event>,
}

impl EventBus {
    /// Create a new event bus
    pub fn new() -> Self {
        let (command_tx, command_rx) = mpsc::channel(100);
        let (event_tx, _) = broadcast::channel(100);
        
        Self {
            command_tx,
            command_rx: Some(command_rx),
            event_tx,
        }
    }
    
    /// Get a command sender
    pub fn command_sender(&self) -> mpsc::Sender<Command> {
        self.command_tx.clone()
    }
    
    /// Take the command receiver (can only be called once)
    pub fn take_command_receiver(&mut self) -> mpsc::Receiver<Command> {
        self.command_rx
            .take()
            .expect("Command receiver already taken")
    }
    
    /// Get an event receiver
    pub fn event_receiver(&self) -> broadcast::Receiver<Event> {
        self.event_tx.subscribe()
    }
    
    /// Get the event sender
    pub fn event_sender(&self) -> broadcast::Sender<Event> {
        self.event_tx.clone()
    }
    
    /// Send a command
    #[instrument(skip(self))]
    pub async fn send_command(&self, command: Command) -> Result<(), mpsc::error::SendError<Command>> {
        debug!("Sending command: {:?}", command);
        self.command_tx.send(command).await
    }
    
    /// Broadcast an event
    #[instrument(skip(self))]
    pub fn send_event(&self, event: Event) -> Result<usize, broadcast::error::SendError<Event>> {
        debug!("Broadcasting event: {:?}", event);
        self.event_tx.send(event)
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use phosphor_common::types::Size;
    
    #[tokio::test]
    async fn test_command_channel() {
        let mut bus = EventBus::new();
        let sender = bus.command_sender();
        let mut receiver = bus.take_command_receiver();
        
        // Send command
        sender.send(Command::Write(b"test".to_vec())).await.unwrap();
        
        // Receive command
        let cmd = receiver.recv().await.unwrap();
        match cmd {
            Command::Write(data) => assert_eq!(data, b"test"),
            _ => panic!("Wrong command type"),
        }
    }
    
    #[tokio::test]
    async fn test_event_broadcast() {
        let bus = EventBus::new();
        let mut receiver1 = bus.event_receiver();
        let mut receiver2 = bus.event_receiver();
        
        // Send event
        bus.send_event(Event::Resized(Size::new(80, 24))).unwrap();
        
        // Both receivers should get it
        let event1 = receiver1.recv().await.unwrap();
        let event2 = receiver2.recv().await.unwrap();
        
        match (event1, event2) {
            (Event::Resized(size1), Event::Resized(size2)) => {
                assert_eq!(size1, Size::new(80, 24));
                assert_eq!(size2, Size::new(80, 24));
            }
            _ => panic!("Wrong event type"),
        }
    }
}