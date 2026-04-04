use tokio::net::TcpListener;
mod background;
mod commands;
mod server;
mod store;
use crate::background::expire_type;
use crate::store::Db;
use server::handle_connection;

#[tokio::main]
async fn main() {
    let listener = match TcpListener::bind("127.0.0.1:8080").await {
        Ok(l) => l,
        Err(e) => {
            println!("Failed to bind the port on 8080:  {}", e);
            return;
        }
    };

    let db: Db = store::create_db();

    tokio::spawn(expire_type::active_expiry(db.clone()));

    println!("Server running on 8080");
    println!("Waiting for connections...");

    loop {
        match listener.accept().await {
            Ok((socket, addr)) => {
                println!("client connected {}", addr);
                let db_clone = db.clone();
                tokio::spawn(async move { handle_connection(socket, addr, &db_clone).await });
            }
            Err(e) => {
                eprintln!("Failed to accept connection: {}", e);
                continue;
            }
        }
    }
}
