use crate::commands::execute_commands;
use crate::store::Db;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;

pub async fn handle_connection(socket: TcpStream, addr: std::net::SocketAddr, db: &Db) {
    let (reader, mut writer) = socket.into_split();
    let mut reader = BufReader::new(reader);

    let mut line = String::new();

    loop {
        if let Err(e) = writer.write_all(b"redis > ").await {
            eprintln!("Failed to write prompt to {}: {}", addr, e);
            return;
        }

        line.clear();
        match reader.read_line(&mut line).await {
            Ok(0) => {
                println!("client disconnected {}", addr);
                return;
            }
            Ok(_) => {}
            Err(e) => {
                println!("Failed to read from {}: {}", addr, e);
                return;
            }
        };

        let message = line.trim();
        let parts: Vec<&str> = message.split_whitespace().collect();

        if parts.is_empty() {
            continue;
        }

        println!("{} says: {:?}", addr, parts);

        let response = execute_commands(&parts, db).await;

        if let Err(e) = writer.write_all(response.as_bytes()).await {
            eprintln!("Failed to write to {}: {}", addr, e);
            return;
        }
    }
}
