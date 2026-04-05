use crate::commands::execute_commands;
use crate::store::{Db, Stats};
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;
use tokio::sync::broadcast;

pub async fn handle_connection(
    socket: TcpStream,
    addr: std::net::SocketAddr,
    db: Db,
    stats: Stats,
    port: u16,
    monitor_tx: Arc<broadcast::Sender<String>>,
) {
    let (reader, mut writer) = socket.into_split();
    let mut reader = BufReader::new(reader);
    let mut line = String::new();

    loop {
        if let Err(e) = writer.write_all(b"Jedis > ").await {
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

        // handle MONITOR before normal command routing
        if parts[0].to_uppercase() == "MONITOR" {
            if let Err(e) = writer.write_all(b"OK entering monitor mode\n").await {
                eprintln!("Failed to write to {}: {}", addr, e);
                return;
            }
            run_monitor(&mut writer, &monitor_tx).await;
            println!("client {} left monitor mode", addr);
            return;
        }

        let response = execute_commands(&parts, &db, &stats, port, &monitor_tx).await;

        if let Err(e) = writer.write_all(response.as_bytes()).await {
            eprintln!("Failed to write to {}: {}", addr, e);
            return;
        }
    }
}

async fn run_monitor(
    writer: &mut tokio::net::tcp::OwnedWriteHalf,
    monitor_tx: &Arc<broadcast::Sender<String>>,
) {
    let mut rx = monitor_tx.subscribe();

    loop {
        match rx.recv().await {
            Ok(msg) => {
                let line = format!("{}\n", msg);
                if writer.write_all(line.as_bytes()).await.is_err() {
                    break;
                }
            }
            Err(broadcast::error::RecvError::Lagged(n)) => {
                // client was too slow — missed n messages
                let msg = format!("WARNING: missed {} commands\n", n);
                if writer.write_all(msg.as_bytes()).await.is_err() {
                    break;
                }
            }
            Err(_) => break,
        }
    }
}
