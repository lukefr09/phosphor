use phosphor_common::error::{PhosphorError, Result};
use portable_pty::MasterPty;
use std::io::{Read, Write};
use tracing::{debug, error, info};

/// Async I/O wrapper for Unix PTY file descriptors
pub struct AsyncPtyIo {
    reader: Box<dyn Read + Send>,
    writer: Box<dyn Write + Send>,
}

impl AsyncPtyIo {
    pub fn new(master: &Box<dyn MasterPty + Send>) -> Result<Self> {
        info!("Creating AsyncPtyIo wrapper");
        
        // Set the master PTY to non-blocking mode before cloning readers
        if let Some(fd) = master.as_raw_fd() {
            unsafe {
                let flags = libc::fcntl(fd, libc::F_GETFL, 0);
                if flags == -1 {
                    error!("Failed to get file descriptor flags");
                } else {
                    let result = libc::fcntl(fd, libc::F_SETFL, flags | libc::O_NONBLOCK);
                    if result == -1 {
                        error!("Failed to set non-blocking mode");
                    } else {
                        info!("Set PTY master to non-blocking mode");
                    }
                }
            }
        } else {
            error!("Could not get raw file descriptor from master PTY");
        }
        
        // Get reader and writer from the master PTY
        let reader = master.try_clone_reader()
            .map_err(|e| {
                error!("Failed to clone reader: {}", e);
                PhosphorError::Pty(format!("Failed to clone reader: {}", e))
            })?;
        debug!("Successfully cloned reader");
        
        let writer = master.take_writer()
            .map_err(|e| {
                error!("Failed to take writer: {}", e);
                PhosphorError::Pty(format!("Failed to take writer: {}", e))
            })?;
        debug!("Successfully took writer");
        
        info!("AsyncPtyIo created successfully");
        Ok(Self { reader, writer })
    }
    
    pub async fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        let buf_len = buf.len();
        let mut reader = std::mem::replace(&mut self.reader, Box::new(std::io::empty()));
        
        // Use spawn_blocking for the blocking read operation
        let result = tokio::task::spawn_blocking(move || {
            let mut temp_buf = vec![0u8; buf_len];
            match reader.read(&mut temp_buf) {
                Ok(n) => Ok((n, temp_buf, reader)),
                Err(e) => Err((e, reader)),
            }
        })
        .await
        .map_err(|e| PhosphorError::Pty(format!("Task join error: {}", e)))?;
        
        match result {
            Ok((n, temp_buf, reader)) => {
                self.reader = reader;
                if n > 0 {
                    buf[..n].copy_from_slice(&temp_buf[..n]);
                    debug!("Read {} bytes from PTY", n);
                }
                Ok(n)
            }
            Err((e, reader)) => {
                self.reader = reader;
                if e.kind() == std::io::ErrorKind::WouldBlock || e.kind() == std::io::ErrorKind::Interrupted {
                    debug!("Read would block or was interrupted, not an error");
                    // Return 0 to indicate no data available right now
                    Ok(0)
                } else {
                    error!("PTY read error: {}", e);
                    Err(e.into())
                }
            }
        }
    }
    
    pub async fn write(&mut self, data: &[u8]) -> Result<usize> {
        info!("AsyncPtyIo write called with {} bytes", data.len());
        if data.len() < 50 {
            info!("Write data: {:?}", String::from_utf8_lossy(data));
        }
        
        let data = data.to_vec();
        let mut writer = std::mem::replace(&mut self.writer, Box::new(std::io::sink()));
        
        // Use spawn_blocking for the blocking write operation
        let result = tokio::task::spawn_blocking(move || {
            debug!("Executing blocking write");
            match writer.write(&data) {
                Ok(n) => {
                    // Ensure data is flushed
                    if let Err(e) = writer.flush() {
                        error!("Failed to flush after write: {}", e);
                    }
                    Ok((n, writer))
                }
                Err(e) => Err((e, writer)),
            }
        })
        .await
        .map_err(|e| PhosphorError::Pty(format!("Task join error: {}", e)))?;
        
        match result {
            Ok((n, writer)) => {
                self.writer = writer;
                info!("Successfully wrote {} bytes to PTY", n);
                Ok(n)
            }
            Err((e, writer)) => {
                self.writer = writer;
                error!("PTY write error: {}", e);
                Err(e.into())
            }
        }
    }
}