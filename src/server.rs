use crate::commands::execute_commands;
use crate::config::SharedConfig;
use crate::resp::parser::parse_command;
use crate::resp::writer;
use crate::store::{Db, Stats};
use std::sync::Arc;
use std::sync::atomic::Ordering;
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
    config: SharedConfig,
) {
    stats.increment_connections();

    // enforce max connections
    {
        let cfg = config.read().await;
        let current = stats.connected_clients.load(Ordering::Relaxed);
        if current > cfg.max_connections {
            let (_, mut writer) = socket.into_split();
            let _ = writer.write_all(b"-ERR max connections reached\r\n").await;
            stats.decrement_connections();
            return;
        }
    }

    let (reader, mut writer) = socket.into_split();
    let mut reader = BufReader::new(reader);

    // check auth requirement at connection time
    let mut authenticated = {
        let cfg = config.read().await;
        !cfg.auth_enabled // if auth off → already authenticated
    };

    loop {
        let parts = match parse_command(&mut reader).await {
            Some(p) if !p.is_empty() => p,
            _ => {
                println!("client disconnected {}", addr);
                break;
            }
        };

        let parts_str: Vec<&str> = parts.iter().map(|s| s.as_str()).collect();

        {
            let cfg = config.read().await;
            if cfg.loglevel == "debug" {
                println!("{} says: {:?}", addr, parts_str);
            }
        }

        let command = parts_str[0].to_uppercase();

        // AUTH — always allowed
        if command == "AUTH" {
            let response = handle_auth(&parts_str, &config, &mut authenticated).await;
            writer.write_all(response.as_bytes()).await.ok();
            continue;
        }

        // PING — allowed without auth so clients can check server is alive
        if command == "PING" && !authenticated {
            writer.write_all(b"+PONG\r\n").await.ok();
            continue;
        }

        // block everything else if not authenticated
        if !authenticated {
            let response = writer::error("NOAUTH Authentication required — use AUTH password");
            writer.write_all(response.as_bytes()).await.ok();
            continue;
        }

        if command == "MONITOR" {
            writer.write_all(b"+OK\r\n").await.ok();
            run_monitor(&mut writer, &monitor_tx).await;
            println!("client {} left monitor mode", addr);
            break;
        }

        let response = execute_commands(&parts_str, &db, &stats, port, &monitor_tx, &config).await;

        if let Err(e) = writer.write_all(response.as_bytes()).await {
            eprintln!("Failed to write to {}: {}", addr, e);
            break;
        }
    }

    stats.decrement_connections();
    println!("client {} fully disconnected", addr);
}

async fn handle_auth(parts: &[&str], config: &SharedConfig, authenticated: &mut bool) -> String {
    if parts.len() != 2 {
        return writer::error("usage: AUTH password");
    }

    let cfg = config.read().await;

    if !cfg.auth_enabled {
        // auth is off — AUTH command succeeds trivially
        *authenticated = true;
        return writer::simple_string("OK");
    }

    match &cfg.password {
        None => {
            *authenticated = true;
            writer::simple_string("OK")
        }
        Some(expected) => {
            if parts[1] == expected {
                *authenticated = true;
                writer::simple_string("OK")
            } else {
                writer::error("WRONGPASS invalid password")
            }
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
