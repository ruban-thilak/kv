//! Background expiry task for proactive key cleanup
//!
//! Real Redis doesn't just rely on lazy expiry - it also runs a background
//! task that periodically samples keys and removes expired ones.
//!
//! This module provides that background cleanup functionality.

use crate::storage::Db;
use std::time::Duration;
use tokio::time::interval;

/// Configuration for the background expiry task
#[derive(Clone, Debug)]
pub struct ExpiryConfig {
    /// How often to run the cleanup (default: 100ms)
    pub interval: Duration,
    /// Maximum keys to check per cycle (default: 20)
    /// Similar to Redis's active expiry which samples keys
    pub batch_size: usize,
}

impl Default for ExpiryConfig {
    fn default() -> Self {
        Self {
            interval: Duration::from_millis(100),
            batch_size: 20,
        }
    }
}

impl ExpiryConfig {
    /// Create a new config with custom interval
    pub fn with_interval(mut self, interval: Duration) -> Self {
        self.interval = interval;
        self
    }

    /// Create a new config with custom batch size
    pub fn with_batch_size(mut self, batch_size: usize) -> Self {
        self.batch_size = batch_size;
        self
    }
}

/// Statistics from the background expiry task
#[derive(Default, Debug)]
pub struct ExpiryStats {
    pub cycles_run: u64,
    pub keys_expired: u64,
}

/// Runs the background expiry task
///
/// This task periodically scans the database and removes expired keys.
/// It uses a sampling approach similar to Redis:
/// - Check up to `batch_size` keys per cycle
/// - If many keys are expired, the next cycle will catch more
///
/// # Arguments
/// * `db` - The database to clean
/// * `config` - Configuration for the expiry behavior
///
/// # Example
/// ```ignore
/// let db = storage::db::new();
/// let config = ExpiryConfig::default();
/// tokio::spawn(run_expiry_task(db.clone(), config));
/// ```
pub async fn run_expiry_task(db: Db, config: ExpiryConfig) {
    let mut ticker = interval(config.interval);
    let mut stats = ExpiryStats::default();

    println!(
        "Background expiry task started (interval: {:?}, batch_size: {})",
        config.interval, config.batch_size
    );

    loop {
        ticker.tick().await;

        let expired_count = cleanup_expired_keys(&db, config.batch_size).await;

        if expired_count > 0 {
            stats.keys_expired += expired_count as u64;
            // Only log when we actually expire keys to reduce noise
            println!(
                "Expiry: removed {} keys (total: {})",
                expired_count, stats.keys_expired
            );
        }

        stats.cycles_run += 1;
    }
}

/// Performs a single cleanup cycle
///
/// Returns the number of keys that were expired and removed.
async fn cleanup_expired_keys(db: &Db, batch_size: usize) -> usize {
    let mut store = db.lock().await;

    // Collect keys to check (up to batch_size)
    // We collect keys first to avoid borrowing issues
    let keys_to_check: Vec<String> = store
        .keys()
        .take(batch_size)
        .cloned()
        .collect();

    let mut expired_count = 0;

    for key in keys_to_check {
        if let Some(entry) = store.get(&key) {
            if entry.is_expired() {
                store.remove(&key);
                expired_count += 1;
            }
        }
    }

    expired_count
}

/// Alternative: Full scan cleanup (use sparingly, blocks longer)
///
/// This removes ALL expired keys in one pass. Useful for maintenance
/// but holds the lock longer than the sampling approach.
pub async fn cleanup_all_expired(db: &Db) -> usize {
    let mut store = db.lock().await;
    let before = store.len();
    store.retain(|_, entry| !entry.is_expired());
    before - store.len()
}
