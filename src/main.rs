use tokio::net::TcpListener;
mod server;
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

    println!("Server running on 8080");
    println!("Waiting for connections...");

    loop {
        match listener.accept().await {
            Ok((socket, addr)) => {
                println!("client connected {}", addr);
                tokio::spawn(handle_connection(socket, addr));
            }
            Err(e) => {
                eprintln!("Failed to accept connection: {}", e);
                continue;
            }
        }
    }
}
