use kv::storage::db::{self, Entry};
use kv::storage::expiry::ExpiryConfig;
use std::time::{Duration, Instant};

#[tokio::test]
async fn test_cleanup_expired_keys() {
    let db = db::new();

    // Add some keys - some expired, some not
    {
        let mut store = db.lock().await;
        // Already expired (in the past)
        store.insert(
            "expired1".to_string(),
            Entry::with_expiry(
                "value1".to_string(),
                Some(Instant::now() - Duration::from_secs(1)),
            ),
        );
        store.insert(
            "expired2".to_string(),
            Entry::with_expiry(
                "value2".to_string(),
                Some(Instant::now() - Duration::from_secs(1)),
            ),
        );
        // Not expired
        store.insert(
            "valid".to_string(),
            Entry::with_expiry(
                "value3".to_string(),
                Some(Instant::now() + Duration::from_secs(100)),
            ),
        );
        // No expiry
        store.insert(
            "permanent".to_string(),
            Entry::with_expiry("value4".to_string(), None),
        );
    }

    // Run the expiry task briefly
    let config = ExpiryConfig::default().with_batch_size(100);

    // Spawn the task and let it run one cycle
    let db_clone = db.clone();
    let handle = tokio::spawn(async move {
        kv::storage::run_expiry_task(db_clone, config).await;
    });

    // Give it time to run at least one cleanup cycle
    tokio::time::sleep(Duration::from_millis(150)).await;
    handle.abort();

    // Verify remaining keys
    let store = db.lock().await;
    assert_eq!(store.len(), 2);
    assert!(store.contains_key("valid"));
    assert!(store.contains_key("permanent"));
}

#[tokio::test]
async fn test_expiry_config_builder() {
    let config = ExpiryConfig::default()
        .with_interval(Duration::from_secs(1))
        .with_batch_size(50);

    assert_eq!(config.interval, Duration::from_secs(1));
    assert_eq!(config.batch_size, 50);
}

#[tokio::test]
async fn test_cleanup_all_expired() {
    let db = db::new();

    // Add 10 expired keys
    {
        let mut store = db.lock().await;
        for i in 0..10 {
            store.insert(
                format!("key{}", i),
                Entry::with_expiry(
                    format!("value{}", i),
                    Some(Instant::now() - Duration::from_secs(1)),
                ),
            );
        }
        // Add one valid key
        store.insert(
            "valid".to_string(),
            Entry::with_expiry("value".to_string(), None),
        );
    }

    // Use cleanup_all_expired
    let removed = kv::storage::expiry::cleanup_all_expired(&db).await;

    assert_eq!(removed, 10);

    let store = db.lock().await;
    assert_eq!(store.len(), 1);
    assert!(store.contains_key("valid"));
}
