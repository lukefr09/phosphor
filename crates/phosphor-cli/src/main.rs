use anyhow::Result;
use clap::Parser;
use crossterm::{
    cursor::{Hide, Show},
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    terminal::{self, Clear, ClearType},
};
use phosphor_common::types::Size;
use phosphor_core::{events::Command, Terminal};
use std::io::{self, Write};
use tokio::sync::mpsc;
use tracing::{debug, error, info};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Parser, Debug)]
#[command(author, version, about = "Phosphor Terminal CLI Test Tool", long_about = None)]
struct Args {
    /// Enable debug logging
    #[arg(short, long)]
    debug: bool,
    
    /// Terminal rows (defaults to current terminal size)
    #[arg(long)]
    rows: Option<u16>,
    
    /// Terminal columns (defaults to current terminal size)
    #[arg(long)]
    cols: Option<u16>,
    
    /// Override shell to use (e.g., /bin/sh, /bin/bash)
    #[arg(long)]
    shell: Option<String>,
    
    /// Use minimal environment (env -i)
    #[arg(long)]
    minimal_env: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    
    // Initialize logging
    let filter = if args.debug {
        "phosphor=debug"
    } else {
        "phosphor=info"
    };
    
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| filter.into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
    
    info!("Starting Phosphor Terminal CLI");
    
    // Get terminal size
    let (width, height) = terminal::size().unwrap_or((80, 24));
    let size = Size::new(
        args.cols.unwrap_or(if width > 0 { width } else { 80 }),
        args.rows.unwrap_or(if height > 0 { height } else { 24 }),
    );
    
    info!("Terminal size: {:?}", size);
    
    // Validate size
    if size.rows == 0 || size.cols == 0 {
        error!("Invalid terminal size detected: {:?}", size);
        return Err(anyhow::anyhow!("Terminal must have non-zero size"));
    }
    
    // Set up terminal
    terminal::enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, Clear(ClearType::All), Hide)?;
    
    // Set shell override if provided
    if let Some(shell) = &args.shell {
        std::env::set_var("SHELL", shell);
        info!("Using shell override: {}", shell);
    }
    
    // Set minimal environment flag
    if args.minimal_env {
        std::env::set_var("PHOSPHOR_MINIMAL_ENV", "1");
        info!("Using minimal environment");
    }
    
    // Create terminal
    let terminal = Terminal::new(size)?;
    let cmd_sender = terminal.command_sender();
    let mut event_receiver = terminal.event_receiver();
    
    // Spawn terminal task
    let terminal_task = tokio::spawn(async move {
        terminal.run().await
    });
    
    // Spawn input handler
    let input_task = tokio::spawn(handle_input(cmd_sender.clone()));
    
    // Spawn event handler
    let event_task = tokio::spawn(async move {
        info!("Event handler started");
        while let Ok(event) = event_receiver.recv().await {
            use phosphor_core::events::Event;
            match event {
                Event::OutputReady(data) => {
                    debug!("Received OutputReady event with {} bytes", data.len());
                    // Write raw output - the terminal emulator has already processed ANSI sequences
                    let mut stdout = io::stdout();
                    if let Err(e) = stdout.write_all(&data) {
                        error!("Failed to write to stdout: {}", e);
                    }
                    if let Err(e) = stdout.flush() {
                        error!("Failed to flush stdout: {}", e);
                    }
                }
                Event::StateChanged => {
                    debug!("Received StateChanged event");
                    // State changes are handled internally
                }
                Event::Closed => {
                    info!("Received Closed event - terminal closed");
                    break;
                }
                _ => {
                    debug!("Received unhandled event");
                }
            }
        }
        info!("Event handler exiting");
    });
    
    // Wait for tasks
    tokio::select! {
        result = terminal_task => {
            info!("Terminal task ended: {:?}", result);
        }
        result = input_task => {
            info!("Input task ended: {:?}", result);
        }
        result = event_task => {
            info!("Event task ended: {:?}", result);
        }
    }
    
    // Cleanup
    execute!(stdout, Show)?;
    terminal::disable_raw_mode()?;
    
    Ok(())
}

async fn handle_input(cmd_sender: mpsc::Sender<Command>) -> Result<()> {
    info!("Input handler started");
    loop {
        if event::poll(std::time::Duration::from_millis(100))? {
            match event::read()? {
                Event::Key(KeyEvent {
                    code: KeyCode::Char('c'),
                    modifiers: KeyModifiers::CONTROL,
                    ..
                }) => {
                    info!("Ctrl+C pressed, closing terminal");
                    cmd_sender.send(Command::Close).await?;
                    break;
                }
                Event::Key(KeyEvent {
                    code: KeyCode::Char(c),
                    modifiers: KeyModifiers::NONE,
                    ..
                }) => {
                    debug!("Key pressed: '{}' (0x{:02x})", c, c as u8);
                    let data = vec![c as u8];
                    cmd_sender.send(Command::Write(data)).await?;
                }
                Event::Key(KeyEvent {
                    code: KeyCode::Char(c),
                    modifiers,
                    ..
                }) if modifiers.contains(KeyModifiers::SHIFT) => {
                    // Handle shifted characters
                    let data = vec![c as u8];
                    cmd_sender.send(Command::Write(data)).await?;
                }
                Event::Key(KeyEvent {
                    code: KeyCode::Enter,
                    ..
                }) => {
                    cmd_sender.send(Command::Write(vec![b'\r'])).await?;
                }
                Event::Key(KeyEvent {
                    code: KeyCode::Tab,
                    ..
                }) => {
                    cmd_sender.send(Command::Write(vec![b'\t'])).await?;
                }
                Event::Key(KeyEvent {
                    code: KeyCode::Backspace,
                    ..
                }) => {
                    cmd_sender.send(Command::Write(vec![0x7f])).await?; // DEL character
                }
                Event::Key(KeyEvent {
                    code: KeyCode::Up,
                    ..
                }) => {
                    // Send cursor up sequence
                    cmd_sender.send(Command::Write(vec![0x1b, b'[', b'A'])).await?;
                }
                Event::Key(KeyEvent {
                    code: KeyCode::Down,
                    ..
                }) => {
                    // Send cursor down sequence
                    cmd_sender.send(Command::Write(vec![0x1b, b'[', b'B'])).await?;
                }
                Event::Key(KeyEvent {
                    code: KeyCode::Right,
                    ..
                }) => {
                    // Send cursor right sequence
                    cmd_sender.send(Command::Write(vec![0x1b, b'[', b'C'])).await?;
                }
                Event::Key(KeyEvent {
                    code: KeyCode::Left,
                    ..
                }) => {
                    // Send cursor left sequence
                    cmd_sender.send(Command::Write(vec![0x1b, b'[', b'D'])).await?;
                }
                Event::Key(KeyEvent {
                    code: KeyCode::Esc,
                    ..
                }) => {
                    // Send ESC
                    cmd_sender.send(Command::Write(vec![0x1b])).await?;
                }
                Event::Resize(cols, rows) => {
                    info!("Terminal resized to {}x{}", cols, rows);
                    cmd_sender.send(Command::Resize(Size::new(cols, rows))).await?;
                }
                _ => {
                    debug!("Unhandled input event");
                }
            }
        }
    }
    
    info!("Input handler exiting");
    Ok(())
}