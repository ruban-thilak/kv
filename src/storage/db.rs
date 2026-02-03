use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use std::time::Instant;

#[derive(Clone, Debug)]
pub struct Entry {
    pub value: String,
    pub expires_at: Option<Instant>,
}

impl Entry {
    pub fn new(value: String, ttl_ms: Option<u64>) -> Self {
        let expires_at = ttl_ms.map(|ms| Instant::now() + std::time::Duration::from_millis(ms));
        Self { value, expires_at }
    }

    pub fn with_expiry(value: String, expires_at: Option<Instant>) -> Self {
        Self { value, expires_at }
    }

    pub fn is_expired(&self) -> bool {
        match self.expires_at {
            Some(at) => Instant::now() > at,
            None => false,
        }
    }
}

/// Thread-safe, async-compatible database type
pub type Db = Arc<Mutex<HashMap<String, Entry>>>;

/// Creates a new empty database instance
pub fn new() -> Db {
    Arc::new(Mutex::new(HashMap::new()))
}
