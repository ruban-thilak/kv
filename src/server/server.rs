use crate::storage::Db;
use crate::server::handler::handle_request;
use tokio::net::TcpListener;

/// Server struct that manages the TCP listener and database instance
pub struct Server {
    listener: TcpListener,
    db: Db,
}

impl Server {
    /// Create a new Server instance
    pub async fn new(addr: &str, db: Db) -> anyhow::Result<Self> {
        let listener = TcpListener::bind(addr).await?;
        println!("KV server running on {}", addr);
        Ok(Self { listener, db })
    }

    /// Run the server loop to accept connections
    pub async fn run(self) -> anyhow::Result<()> {
        loop {
            let (socket, addr) = self.listener.accept().await?;
            println!("New connection from {}", addr);

            let db = self.db.clone();
            tokio::spawn(async move {
                if let Err(e) = handle_request(socket, db).await {
                    eprintln!("Error handling request: {}", e);
                }
            });
        }
    }
}
