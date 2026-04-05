use tokio::net::TcpListener;
mod background;
mod commands;
mod config;
mod server;
mod store;
use crate::background::expire_type;
use crate::store::{Db, Stats};
use config::Config;
use server::handle_connection;
use std::sync::Arc;
use tokio::sync::broadcast;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    let config = Config::new();

    let listener = match TcpListener::bind(config.address()).await {
        Ok(l) => l,
        Err(e) => {
            println!("Failed to bind the port on 8080:  {}", e);
            return;
        }
    };

    let db: Db = store::create_db();
    let stats: Stats = store::create_stats();
    let (moniter_tx, _) = broadcast::channel::<String>(100);
    let moniter_tx = Arc::new(moniter_tx);

    tokio::spawn(expire_type::active_expiry(db.clone(), stats.clone()));

    println!("Server running on {}", config.port);
    println!("Waiting for connections...");
    println!("\n\"");
    println!("\n\"");

    loop {
        match listener.accept().await {
            Ok((socket, addr)) => {
                println!("client connected {}", addr);
                let db_clone = db.clone();
                let stats_clone = stats.clone();
                let moniter_clone = moniter_tx.clone();
                stats.increment_connections();
                tokio::spawn(async move {
                    handle_connection(
                        socket,
                        addr,
                        db_clone,
                        stats_clone.clone(),
                        config.port,
                        moniter_clone,
                    )
                    .await;
                    stats_clone.decrement_connections();
                });
            }
            Err(e) => {
                eprintln!("Failed to accept connection: {}", e);
                continue;
            }
        }
    }
}
