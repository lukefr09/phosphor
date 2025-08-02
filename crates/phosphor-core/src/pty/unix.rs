use phosphor_common::error::{PhosphorError, Result};
use portable_pty::MasterPty;
use std::io::{Read, Write};
use std::sync::{Arc, Mutex};
use tracing::{debug, error, info};

/// Async I/O wrapper for Unix PTY file descriptors
pub struct AsyncPtyIo {
    reader: Arc<Mutex<Box<dyn Read + Send>>>,
    writer: Arc<Mutex<Box<dyn Write + Send>>>,
}

impl AsyncPtyIo {
    pub fn new(master: &Box<dyn MasterPty + Send>) -> Result<Self> {
        info!("Creating AsyncPtyIo wrapper");
        
        // Get reader and writer from the master PTY
        // Note: We're keeping blocking I/O - no O_NONBLOCK
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
        Ok(Self { 
            reader: Arc::new(Mutex::new(reader)),
            writer: Arc::new(Mutex::new(writer)),
        })
    }
    
    pub async fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        let buf_len = buf.len();
        let reader = Arc::clone(&self.reader);
        
        // Use spawn_blocking for the blocking read operation
        let result = tokio::task::spawn_blocking(move || {
            let mut temp_buf = vec![0u8; buf_len];
            
            // Lock the reader for the duration of the read
            let mut reader_guard = reader.lock().unwrap();
            match reader_guard.read(&mut temp_buf) {
                Ok(n) => Ok((n, temp_buf)),
                Err(e) => Err(e),
            }
        })
        .await
        .map_err(|e| PhosphorError::Pty(format!("Task join error: {}", e)))?;
        
        match result {
            Ok((n, temp_buf)) => {
                if n > 0 {
                    buf[..n].copy_from_slice(&temp_buf[..n]);
                    debug!("Read {} bytes from PTY", n);
                }
                Ok(n)
            }
            Err(e) => {
                error!("PTY read error: {}", e);
                Err(e.into())
            }
        }
    }
    
    pub async fn write(&mut self, data: &[u8]) -> Result<usize> {
        info!("AsyncPtyIo write called with {} bytes", data.len());
        if data.len() < 50 {
            info!("Write data: {:?}", String::from_utf8_lossy(data));
        }
        
        let data = data.to_vec();
        let writer = Arc::clone(&self.writer);
        
        // Use spawn_blocking for the blocking write operation
        let result = tokio::task::spawn_blocking(move || {
            debug!("Executing blocking write");
            
            // Lock the writer for the duration of the write
            let mut writer_guard = writer.lock().unwrap();
            match writer_guard.write(&data) {
                Ok(n) => {
                    // Ensure data is flushed
                    if let Err(e) = writer_guard.flush() {
                        error!("Failed to flush after write: {}", e);
                    }
                    Ok(n)
                }
                Err(e) => Err(e),
            }
        })
        .await
        .map_err(|e| PhosphorError::Pty(format!("Task join error: {}", e)))?;
        
        match result {
            Ok(n) => {
                info!("Successfully wrote {} bytes to PTY", n);
                Ok(n)
            }
            Err(e) => {
                error!("PTY write error: {}", e);
                Err(e.into())
            }
        }
    }
}