use async_trait::async_trait;
use phosphor_common::{error::{PhosphorError, Result}, traits::TerminalBackend, types::Size};
use portable_pty::{native_pty_system, CommandBuilder, MasterPty, PtySize};
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{debug, error, info, instrument};

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
        info!("Starting PTY spawn_shell with size: {:?}", size);
        
        let pty_system = native_pty_system();
        let pty_size = PtySize {
            rows: size.rows,
            cols: size.cols,
            pixel_width: 0,
            pixel_height: 0,
        };
        
        debug!("Opening PTY with size {:?}", pty_size);
        let pair = pty_system.openpty(pty_size)
            .map_err(|e| {
                error!("Failed to open PTY: {}", e);
                PhosphorError::Pty(format!("Failed to open PTY: {}", e))
            })?;
        info!("PTY opened successfully");
        
        // Determine shell to spawn
        let shell = std::env::var("SHELL").unwrap_or_else(|_| {
            if cfg!(windows) {
                "cmd.exe".to_string()
            } else {
                "/bin/sh".to_string()
            }
        });
        
        info!("Spawning shell: {}", shell);
        
        // Check if we should use minimal environment
        let use_minimal_env = std::env::var("PHOSPHOR_MINIMAL_ENV").is_ok();
        
        let mut cmd = if use_minimal_env {
            info!("Using minimal environment with env -i");
            let mut env_cmd = CommandBuilder::new("env");
            env_cmd.arg("-i");
            env_cmd.arg(format!("PATH={}", std::env::var("PATH").unwrap_or_else(|_| "/usr/local/bin:/usr/bin:/bin".to_string())));
            env_cmd.arg("TERM=xterm-256color");
            env_cmd.arg("HOME=/tmp");
            env_cmd.arg("USER=user");
            env_cmd.arg(&shell);
            env_cmd
        } else {
            CommandBuilder::new(&shell)
        };
        
        // Force interactive mode and bypass config files
        // Check if it's bash or zsh - they need different flags
        if shell.contains("bash") && !use_minimal_env {
            cmd.arg("--noprofile");  // Skip /etc/profile and ~/.profile
            cmd.arg("--norc");       // Skip ~/.bashrc
            cmd.arg("-i");           // Interactive mode
            info!("Added --noprofile --norc -i flags for bash");
        } else if shell.contains("zsh") && !use_minimal_env {
            cmd.arg("--no-rcs");     // Skip all rc files
            cmd.arg("-i");           // Interactive mode
            info!("Added --no-rcs -i flags for zsh");
        } else if shell.contains("sh") && !use_minimal_env {
            // POSIX sh doesn't always support -i but we can try
            cmd.arg("-i");
            info!("Added -i flag for sh (may not be supported)");
        }
        
        // Set up environment for interactive shell (unless using minimal env)
        if !use_minimal_env {
            cmd.env("TERM", "xterm-256color");
            cmd.env("COLORTERM", "truecolor");
            cmd.env("PS1", "\\u@\\h:\\w\\$ ");  // Set a proper prompt
            cmd.env("SHELL", &shell);  // Ensure SHELL is set
            cmd.env("USER", std::env::var("USER").unwrap_or_else(|_| "user".to_string()));
            cmd.env("HOME", std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string()));
            cmd.env("PATH", std::env::var("PATH").unwrap_or_else(|_| "/usr/local/bin:/usr/bin:/bin".to_string()));
        }
        
        // Set current directory
        if let Ok(cwd) = std::env::current_dir() {
            cmd.cwd(cwd);
        }
        
        // Ensure the PTY will be the controlling terminal
        // This is the default, but let's be explicit
        cmd.set_controlling_tty(true);
        
        debug!("Environment configured for interactive shell");
        
        // Configure slave PTY before spawning
        // Note: portable-pty should handle basic TTY setup, but we'll log it
        info!("Spawning command on slave PTY with controlling terminal");
        
        let mut child = pair.slave.spawn_command(cmd)
            .map_err(|e| {
                error!("Failed to spawn shell '{}': {}", shell, e);
                PhosphorError::Pty(format!("Failed to spawn shell: {}", e))
            })?;
        info!("Shell process spawned successfully");
        
        // IMPORTANT: Drop the slave to relinquish it to the child
        drop(pair.slave);
        info!("Dropped slave PTY handle");
        
        // Give the shell a moment to initialize
        std::thread::sleep(std::time::Duration::from_millis(50));
        
        // Check if the process is still alive after spawn
        match child.try_wait() {
            Ok(None) => info!("Shell process is running after spawn"),
            Ok(Some(status)) => {
                error!("Shell exited immediately after spawn with status: {:?}", status);
                return Err(PhosphorError::Pty(format!("Shell exited immediately: {:?}", status)));
            }
            Err(e) => error!("Error checking shell status after spawn: {}", e),
        }
            
        // Create async I/O wrapper
        debug!("Creating async I/O wrapper");
        let io = AsyncPtyIo::new(&pair.master)?;
        info!("Async I/O wrapper created");
        
        let inner = PtyManagerInner {
            master: pair.master,
            io,
            child,
        };
        
        info!("PtyManager initialized successfully");
        Ok(Self {
            inner: Arc::new(Mutex::new(inner)),
        })
    }
}

#[async_trait]
impl TerminalBackend for PtyManager {
    #[instrument(skip(self, data))]
    async fn write(&mut self, data: &[u8]) -> Result<usize> {
        debug!("PTY write called with {} bytes", data.len());
        let mut inner = self.inner.lock().await;
        match inner.io.write(data).await {
            Ok(n) => {
                debug!("PTY write successful: {} bytes written", n);
                Ok(n)
            }
            Err(e) => {
                error!("PTY write error: {}", e);
                Err(e)
            }
        }
    }
    
    #[instrument(skip(self, buf))]
    async fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        debug!("PTY read called with buffer size: {}", buf.len());
        let mut inner = self.inner.lock().await;
        match inner.io.read(buf).await {
            Ok(0) => {
                info!("PTY read returned 0 bytes (EOF)");
                Ok(0)
            }
            Ok(n) => {
                debug!("PTY read successful: {} bytes read", n);
                if n < 50 {
                    debug!("PTY read data: {:?}", String::from_utf8_lossy(&buf[..n]));
                }
                Ok(n)
            }
            Err(e) => {
                error!("PTY read error: {}", e);
                Err(e)
            }
        }
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
            Ok(None) => {
                debug!("PTY process is still running");
                true  // Still running
            }
            Ok(Some(status)) => {
                error!("PTY process exited with status: {:?}", status);
                false
            }
            Err(e) => {
                error!("Error checking PTY process status: {}", e);
                false
            }
        }
    }
}