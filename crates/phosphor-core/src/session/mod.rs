use phosphor_common::{error::Result, types::Size};
use std::sync::atomic::{AtomicU64, Ordering};
use tokio::sync::RwLock;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

static SESSION_COUNTER: AtomicU64 = AtomicU64::new(0);

/// Session identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SessionId(u64);

impl SessionId {
    /// Create a new unique session ID
    pub fn new() -> Self {
        Self(SESSION_COUNTER.fetch_add(1, Ordering::SeqCst))
    }
}

impl std::fmt::Display for SessionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "session-{}", self.0)
    }
}

/// Session metadata
#[derive(Debug, Clone)]
pub struct SessionInfo {
    pub id: SessionId,
    pub title: String,
    pub created_at: u64,
    pub size: Size,
    pub working_directory: Option<String>,
}

impl SessionInfo {
    pub fn new(title: String, size: Size) -> Self {
        let created_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
            
        Self {
            id: SessionId::new(),
            title,
            created_at,
            size,
            working_directory: std::env::current_dir()
                .ok()
                .and_then(|p| p.to_str().map(String::from)),
        }
    }
}

/// Basic session manager (to be expanded in later phases)
pub struct SessionManager {
    sessions: Arc<RwLock<Vec<SessionInfo>>>,
}

impl SessionManager {
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(RwLock::new(Vec::new())),
        }
    }
    
    pub async fn create_session(&self, title: String, size: Size) -> Result<SessionInfo> {
        let session = SessionInfo::new(title, size);
        let mut sessions = self.sessions.write().await;
        sessions.push(session.clone());
        Ok(session)
    }
    
    pub async fn list_sessions(&self) -> Vec<SessionInfo> {
        self.sessions.read().await.clone()
    }
    
    pub async fn remove_session(&self, id: SessionId) -> Result<()> {
        let mut sessions = self.sessions.write().await;
        sessions.retain(|s| s.id != id);
        Ok(())
    }
}

impl Default for SessionManager {
    fn default() -> Self {
        Self::new()
    }
}