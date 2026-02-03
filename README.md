# KV - Lightweight Async Key-Value Store

A simple, fast, TCP-based key-value store written in Rust with async I/O.

## 🚀 Quick Start

```bash
# Run the server
cargo run

# Connect with netcat
nc localhost 6969
```

## 📝 Supported Commands

| Command | Description | Example |
|---------|-------------|---------|
| `PING` | Health check | `PING` → `PONG` |
| `SET key value` | Store a value | `SET name Alice` → `OK` |
| `GET key` | Retrieve a value | `GET name` → `Alice` |
| `DEL key` | Delete a key | `DEL name` → `(integer) 1` |
| `KEYS` | List all keys | `KEYS` → `1) "name"` |

## 📁 Project Structure

```
src/
├── main.rs          # Entry point
├── lib.rs           # Library root
├── server/          # 🌐 Networking layer
│   ├── mod.rs
│   └── handler.rs   # TCP request handling
├── storage/         # 💾 Storage engine
│   ├── mod.rs
│   └── db.rs        # Database implementation
└── protocol/        # 📜 Command processing
    ├── mod.rs
    └── command.rs   # Command parser & executor
```

## 🛠️ Development

```bash
# Build
cargo build

# Run tests
cargo test

# Run with logging
RUST_LOG=debug cargo run
```

## 📜 License

MIT
