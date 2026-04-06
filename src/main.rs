use tokio::net::TcpListener;
mod background;
mod commands;
mod config;
mod resp;
mod server;
mod store;
use crate::background::expire_type;
use crate::config::{Config, create_shared_config};
use crate::store::{Db, Stats};
use server::handle_connection;
use std::sync::Arc;
use tokio::sync::broadcast;

#[tokio::main]
async fn main() {
    let config = match Config::new() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Config error: {}", e);
            std::process::exit(1);
        }
    };

    let address = config.address();
    let port = config.port;
    let mbuffer = config.monitor_buffer;

    // create shared config — this is the ONLY config used after startup
    let shared_config = create_shared_config(config);

    let listener = match TcpListener::bind(&address).await {
        Ok(l) => l,
        Err(e) => {
            println!("Failed to bind the port on 8080:  {}", e);
            return;
        }
    };

    let db: Db = store::create_db();
    let stats: Stats = store::create_stats();
    let (moniter_tx, _) = broadcast::channel::<String>(mbuffer);
    let moniter_tx = Arc::new(moniter_tx);

    tokio::spawn(expire_type::active_expiry(db.clone(), stats.clone()));

    println!("Server running on {}", port);
    println!("Waiting for connections...");

    loop {
        match listener.accept().await {
            Ok((socket, addr)) => {
                println!("client connected {}", addr);
                let db_clone = db.clone();
                let stats_clone = stats.clone();
                let moniter_clone = moniter_tx.clone();
                let config_clone = shared_config.clone();
                tokio::spawn(async move {
                    handle_connection(
                        socket,
                        addr,
                        db_clone,
                        stats_clone.clone(),
                        port,
                        moniter_clone,
                        config_clone,
                    )
                    .await;
                });
            }
            Err(e) => {
                eprintln!("Failed to accept connection: {}", e);
                continue;
            }
        }
    }
}
