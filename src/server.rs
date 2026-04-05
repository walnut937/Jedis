use crate::commands::execute_commands;
use crate::resp::parser::parse_command;
use crate::store::{Db, Stats};
use std::sync::Arc;
use tokio::io::{AsyncWriteExt, BufReader};
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
    // increment HERE inside the task — not in main
    stats.increment_connections();

    let (reader, mut writer) = socket.into_split();
    let mut reader = BufReader::new(reader);

    loop {
        let parts = match parse_command(&mut reader).await {
            Some(p) if !p.is_empty() => p,
            _ => {
                println!("client disconnected {}", addr);
                break; // always hits decrement at bottom
            }
        };

        let parts_str: Vec<&str> = parts.iter().map(|s| s.as_str()).collect();
        println!("{} says: {:?}", addr, parts_str);

        if parts_str[0].to_uppercase() == "MONITOR" {
            writer.write_all(b"+OK\r\n").await.unwrap();
            run_monitor(&mut writer, &monitor_tx).await;
            println!("client {} left monitor mode", addr);
            break; // break not return — hits decrement at bottom
        }

        let response = execute_commands(&parts_str, &db, &stats, port, &monitor_tx).await;

        if let Err(e) = writer.write_all(response.as_bytes()).await {
            eprintln!("Failed to write to {}: {}", addr, e);
            break; // break not return — hits decrement at bottom
        }
    }

    // always runs — no matter how the loop exits
    stats.decrement_connections();
    println!("client {} fully disconnected", addr);
}

async fn run_monitor(
    writer: &mut tokio::net::tcp::OwnedWriteHalf,
    monitor_tx: &Arc<broadcast::Sender<String>>,
) {
    let mut rx = monitor_tx.subscribe();

    loop {
        match rx.recv().await {
            Ok(msg) => {
                let line = format!("+{}\r\n", msg);
                if writer.write_all(line.as_bytes()).await.is_err() {
                    break;
                }
            }
            Err(broadcast::error::RecvError::Lagged(n)) => {
                let msg = format!("-WARNING missed {} commands\r\n", n);
                if writer.write_all(msg.as_bytes()).await.is_err() {
                    break;
                }
            }
            Err(_) => break,
        }
    }
}
