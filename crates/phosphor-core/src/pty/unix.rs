use phosphor_common::error::{PhosphorError, Result};
use portable_pty::MasterPty;
use std::io::{Read, Write};
use tracing::debug;

/// Async I/O wrapper for Unix PTY file descriptors
pub struct AsyncPtyIo {
    reader: Box<dyn Read + Send>,
    writer: Box<dyn Write + Send>,
}

impl AsyncPtyIo {
    pub fn new(master: &Box<dyn MasterPty + Send>) -> Result<Self> {
        // Get reader and writer from the master PTY
        let reader = master.try_clone_reader()
            .map_err(|e| PhosphorError::Pty(format!("Failed to clone reader: {}", e)))?;
        let writer = master.take_writer()
            .map_err(|e| PhosphorError::Pty(format!("Failed to take writer: {}", e)))?;
        
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
                if e.kind() == std::io::ErrorKind::WouldBlock {
                    Ok(0)
                } else {
                    Err(e.into())
                }
            }
        }
    }
    
    pub async fn write(&mut self, data: &[u8]) -> Result<usize> {
        let data = data.to_vec();
        let mut writer = std::mem::replace(&mut self.writer, Box::new(std::io::sink()));
        
        // Use spawn_blocking for the blocking write operation
        let result = tokio::task::spawn_blocking(move || {
            match writer.write(&data) {
                Ok(n) => Ok((n, writer)),
                Err(e) => Err((e, writer)),
            }
        })
        .await
        .map_err(|e| PhosphorError::Pty(format!("Task join error: {}", e)))?;
        
        match result {
            Ok((n, writer)) => {
                self.writer = writer;
                debug!("Wrote {} bytes to PTY", n);
                Ok(n)
            }
            Err((e, writer)) => {
                self.writer = writer;
                Err(e.into())
            }
        }
    }
}