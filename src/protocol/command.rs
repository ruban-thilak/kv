use crate::storage::{Db, db::Entry};
use std::time::Instant;

/// Processes a command string and returns the response
pub async fn process_command(line: &str, db: &Db) -> String {
    let parts: Vec<&str> = line.trim().split_whitespace().collect();

    if parts.is_empty() {
        return "ERROR: Empty command\n".to_string();
    }

    match parts[0].to_uppercase().as_str() {
        "GET" => {
            if parts.len() < 2 {
                return "ERROR: GET requires a key\n".to_string();
            }
            let key = parts[1];
            let mut store = db.lock().await;

            // Check if key exists and handle expiration lazily
            if let Some(entry) = store.get(key) {
                if entry.is_expired() {
                    store.remove(key);
                    "(nil)\n".to_string()
                } else {
                    format!("{}\n", entry.value)
                }
            } else {
                "(nil)\n".to_string()
            }
        }
        "SET" => {
            if parts.len() < 3 {
                return "ERROR: SET requires a key and value\n".to_string();
            }
            let key = parts[1].to_string();

            // Parse optional EX argument: SET key value [EX seconds]
            let mut expires_at: Option<Instant> = None;
            let mut value_end_index = parts.len();

            // Look for EX option (case-insensitive) starting from the 4th part (index 3)
            if let Some(ex_pos) = parts.iter().skip(3).position(|&p| p.eq_ignore_ascii_case("EX")) {
                let ex_index = ex_pos + 3; // Adjust for skip(3) to get actual index in `parts`

                // Check if there's a seconds argument after EX
                if ex_index + 1 < parts.len() {
                    match parts[ex_index + 1].parse::<u64>() {
                        Ok(seconds) => {
                            expires_at = Some(Instant::now() + std::time::Duration::from_secs(seconds));
                            value_end_index = ex_index; // Value ends before EX
                        }
                        Err(_) => return "ERROR: Invalid EX value\n".to_string(),
                    }
                } else {
                    return "ERROR: EX requires seconds argument\n".to_string();
                }
            }

            let value = parts[2..value_end_index].join(" ");

            let mut store = db.lock().await;
            store.insert(key, Entry::with_expiry(value, expires_at));
            "OK\n".to_string()
        }
        "EXPIRE" => {
            if parts.len() < 3 {
                return "ERROR: EXPIRE requires a key and seconds\n".to_string();
            }
            let key = parts[1];
            match parts[2].parse::<u64>() {
                Ok(seconds) => {
                    let mut store = db.lock().await;
                    if let Some(entry) = store.get_mut(key) {
                        if entry.is_expired() {
                            store.remove(key);
                            "(integer) 0\n".to_string()
                        } else {
                            entry.expires_at = Some(Instant::now() + std::time::Duration::from_secs(seconds));
                            "(integer) 1\n".to_string()
                        }
                    } else {
                        "(integer) 0\n".to_string()
                    }
                }
                Err(_) => "ERROR: Invalid seconds\n".to_string(),
            }
        }
        "TTL" => {
            if parts.len() < 2 {
                return "ERROR: TTL requires a key\n".to_string();
            }
            let key = parts[1];
            let mut store = db.lock().await;

            if let Some(entry) = store.get(key) {
                if entry.is_expired() {
                    store.remove(key);
                    "(integer) -2\n".to_string()
                } else {
                    match entry.expires_at {
                        Some(at) => {
                            let now = Instant::now();
                            if now >= at {
                                store.remove(key);
                                "(integer) -2\n".to_string()
                            } else {
                                let duration = at.duration_since(now);
                                format!("(integer) {}\n", duration.as_secs())
                            }
                        }
                        None => "(integer) -1\n".to_string(),
                    }
                }
            } else {
                "(integer) -2\n".to_string()
            }
        }
        "INCR" => {
            if parts.len() < 2 {
                return "ERROR: INCR requires a key\n".to_string();
            }
            let key = parts[1];
            let mut store = db.lock().await;

            // Check if key exists
            if let Some(entry) = store.get(key) {
                // Check if expired
                if entry.is_expired() {
                    store.remove(key);
                    // Treat as new key, set to 1
                    store.insert(key.to_string(), Entry::with_expiry("1".to_string(), None));
                    "(integer) 1\n".to_string()
                } else {
                    // Try to parse as integer
                    match entry.value.parse::<i64>() {
                        Ok(num) => {
                            let new_value = num + 1;
                            let expires_at = entry.expires_at; // Preserve existing TTL
                            store.insert(key.to_string(), Entry::with_expiry(new_value.to_string(), expires_at));
                            format!("(integer) {}\n", new_value)
                        }
                        Err(_) => "ERROR: value is not an integer\n".to_string(),
                    }
                }
            } else {
                // Key doesn't exist, initialize to 1
                store.insert(key.to_string(), Entry::with_expiry("1".to_string(), None));
                "(integer) 1\n".to_string()
            }
        }
        "DEL" => {
            if parts.len() < 2 {
                return "ERROR: DEL requires a key\n".to_string();
            }
            let key = parts[1];
            let mut store = db.lock().await;
            match store.remove(key) {
                Some(_) => "(integer) 1\n".to_string(),
                None => "(integer) 0\n".to_string(),
            }
        }
        "PING" => "PONG\n".to_string(),
        "KEYS" => {
            let mut store = db.lock().await;

            // Filter out expired keys
            store.retain(|_, v| !v.is_expired());

            if store.is_empty() {
                "(empty list)\n".to_string()
            } else {
                let keys: Vec<&String> = store.keys().collect();
                keys.iter()
                    .enumerate()
                    .map(|(i, k)| format!("{}) \"{}\"\n", i + 1, k))
                    .collect()
            }
        }
        _ => format!("ERROR: Unknown command '{}'\n", parts[0]),
    }
}
