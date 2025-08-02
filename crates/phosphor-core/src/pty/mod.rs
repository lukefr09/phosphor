use async_trait::async_trait;
use phosphor_common::{error::{PhosphorError, Result}, traits::TerminalBackend, types::Size};
use portable_pty::{native_pty_system, CommandBuilder, MasterPty, PtySize};
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{debug, instrument};

#[cfg(unix)]
mod unix;

#[cfg(windows)]
mod windows;

/// Platform-specific file descriptor wrapper
#[cfg(unix)]
use unix::AsyncPtyIo;

#[cfg(windows)]
use windows::AsyncPtyIo;

/// PTY manager that handles process spawning and I/O
#[derive(Clone)]
pub struct PtyManager {
    inner: Arc<Mutex<PtyManagerInner>>,
}

struct PtyManagerInner {
    master: Box<dyn MasterPty + Send>,
    io: AsyncPtyIo,
    #[allow(dead_code)]
    child: Box<dyn portable_pty::Child + Send + Sync>,
}

impl PtyManager {
    /// Spawn a shell process with the given terminal size
    #[instrument]
    pub fn spawn_shell(size: Size) -> Result<Self> {
        let pty_system = native_pty_system();
        let pty_size = PtySize {
            rows: size.rows,
            cols: size.cols,
            pixel_width: 0,
            pixel_height: 0,
        };
        
        debug!("Opening PTY with size {:?}", pty_size);
        let pair = pty_system.openpty(pty_size)
            .map_err(|e| PhosphorError::Pty(format!("Failed to open PTY: {}", e)))?;
        
        // Determine shell to spawn
        let shell = std::env::var("SHELL").unwrap_or_else(|_| {
            if cfg!(windows) {
                "cmd.exe".to_string()
            } else {
                "/bin/sh".to_string()
            }
        });
        
        debug!("Spawning shell: {}", shell);
        let mut cmd = CommandBuilder::new(&shell);
        
        // Set up environment
        cmd.env("TERM", "xterm-256color");
        cmd.env("COLORTERM", "truecolor");
        
        let child = pair.slave.spawn_command(cmd)
            .map_err(|e| PhosphorError::Pty(format!("Failed to spawn shell: {}", e)))?;
            
        // Create async I/O wrapper
        let io = AsyncPtyIo::new(&pair.master)?;
        
        let inner = PtyManagerInner {
            master: pair.master,
            io,
            child,
        };
        
        Ok(Self {
            inner: Arc::new(Mutex::new(inner)),
        })
    }
}

#[async_trait]
impl TerminalBackend for PtyManager {
    #[instrument(skip(self, data))]
    async fn write(&mut self, data: &[u8]) -> Result<usize> {
        let mut inner = self.inner.lock().await;
        inner.io.write(data).await
    }
    
    #[instrument(skip(self, buf))]
    async fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        let mut inner = self.inner.lock().await;
        inner.io.read(buf).await
    }
    
    #[instrument(skip(self))]
    async fn resize(&mut self, size: Size) -> Result<()> {
        let inner = self.inner.lock().await;
        let pty_size = PtySize {
            rows: size.rows,
            cols: size.cols,
            pixel_width: 0,
            pixel_height: 0,
        };
        
        inner.master.resize(pty_size)
            .map_err(|e| PhosphorError::Pty(format!("Failed to resize PTY: {}", e)))?;
            
        debug!("PTY resized to {:?}", size);
        Ok(())
    }
    
    async fn is_alive(&self) -> bool {
        let mut inner = self.inner.lock().await;
        match inner.child.try_wait() {
            Ok(None) => true,  // Still running
            _ => false,        // Exited or error
        }
    }
}