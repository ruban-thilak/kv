//! KV - A lightweight, async key-value store
//!
//! This crate provides a simple TCP-based key-value store with
//! support for basic commands like GET, SET, DEL, PING, and KEYS.
//!
//! ## Expiry Strategies
//!
//! This KV store supports two expiry strategies:
//!
//! 1. **Lazy Expiry**: Keys are checked and removed when accessed (GET, TTL, etc.)
//! 2. **Background Expiry**: A background task proactively removes expired keys
//!
//! Both strategies work together to ensure expired keys don't consume memory.

pub mod protocol;
pub mod server;
pub mod storage;

// Re-export common types for convenience
pub use server::{handle_request, Server};
pub use storage::{Db, ExpiryConfig, run_expiry_task};
