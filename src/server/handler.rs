use crate::protocol::process_command;
use crate::storage::Db;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;

/// Handles an incoming TCP connection
pub async fn handle_request(stream: TcpStream, db: Db) -> anyhow::Result<()> {
    let (reader, mut writer) = stream.into_split();
    let reader = BufReader::new(reader);
    let mut lines = reader.lines();

    while let Some(line) = lines.next_line().await? {
        let response = process_command(&line, &db).await;
        writer.write_all(response.as_bytes()).await?;
    }

    Ok(())
}
