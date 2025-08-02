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
use tracing::{debug, info};
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
    let (width, height) = terminal::size()?;
    let size = Size::new(
        args.cols.unwrap_or(width),
        args.rows.unwrap_or(height),
    );
    
    info!("Terminal size: {:?}", size);
    
    // Set up terminal
    terminal::enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, Clear(ClearType::All), Hide)?;
    
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
        while let Ok(event) = event_receiver.recv().await {
            use phosphor_core::events::Event;
            match event {
                Event::OutputReady(data) => {
                    // In a real implementation, we'd render to screen
                    // For now, just write to stdout
                    let mut stdout = io::stdout();
                    stdout.write_all(&data).ok();
                    stdout.flush().ok();
                }
                Event::StateChanged => {
                    debug!("Terminal state changed");
                }
                Event::Closed => {
                    info!("Terminal closed");
                    break;
                }
                _ => {}
            }
        }
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
                    let data = vec![c as u8];
                    cmd_sender.send(Command::Write(data)).await?;
                }
                Event::Key(KeyEvent {
                    code: KeyCode::Enter,
                    ..
                }) => {
                    cmd_sender.send(Command::Write(vec![b'\n'])).await?;
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
                Event::Resize(cols, rows) => {
                    info!("Terminal resized to {}x{}", cols, rows);
                    cmd_sender.send(Command::Resize(Size::new(rows, cols))).await?;
                }
                _ => {}
            }
        }
    }
    
    Ok(())
}