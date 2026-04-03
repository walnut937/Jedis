use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;

pub async fn handle_connection(socket: TcpStream, addr: std::net::SocketAddr) {
    let (reader, mut writer) = socket.into_split();
    let mut reader = BufReader::new(reader);

    let mut line = String::new();

    loop {
        line.clear();

        match reader.read_line(&mut line).await {
            Ok(0) => {
                println!("client disconnected {}", addr);
                return;
            }
            Ok(n) => n,
            Err(e) => {
                println!("Failed to read form {}: {}", addr, e);
                return;
            }
        };

        let message = line.trim();
        println!("{} says: {}", addr, message);

        let response = match message {
            "PING" => "PONG\n",
            _ => "UNKNOWN\n",
        };

        if let Err(e) = writer.write_all(response.as_bytes()).await {
            eprintln!("Failed to write to {}: {}", addr, e);
        }
    }
}
