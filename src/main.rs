use kv::{storage, Server, Db};
use kv::storage::{run_expiry_task, ExpiryConfig};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let db: Db = storage::db::new();

    // Spawn background expiry task
    // This runs alongside the server, proactively cleaning expired keys
    let expiry_config = ExpiryConfig::default();
    tokio::spawn(run_expiry_task(db.clone(), expiry_config));

    let server = Server::new("127.0.0.1:6969", db).await?;
    server.run().await
}
