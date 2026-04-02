use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    let listener = match TcpListener::bind("127.0.0.1:8080").await {
        Ok(l) => l,
        Err(e) => {
            println!("Failed to bind the port on 8080:  {}", e);
            return;
        }
    };

    loop {
        match listener.accept().await {
            Ok((mut socket, addr)) => {
                println!("Client connected {}", addr);
                tokio::spawn(async move {
                    let mut buffer = vec![0; 1024];

                    let n = match socket.read(&mut buffer).await {
                        Ok(0) => {
                            println!("client disconnected {}", addr);
                            return;
                        }
                        Ok(n) => n,
                        Err(e) => {
                            println!("Failed to read from {}: {}", addr, e);
                            return;
                        }
                    };
                    let message = String::from_utf8_lossy(&buffer[..n]).trim().to_string();
                    println!("{} says: {}", addr, message);

                    let response = if message == "PING" {
                        "PONG\n"
                    } else {
                        "UNKNOWN\n"
                    };

                    if let Err(e) = socket.write_all(response.as_bytes()).await {
                        eprintln!("Failed to write to {}: {}", addr, e);
                    }
                });
            }
            Err(e) => {
                eprintln!("Failed to accept connection: {}", e);
                continue;
            }
        }
    }
}
