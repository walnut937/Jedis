use crate::commands::hashmap;
use crate::commands::string;
use crate::store::Db;

pub async fn execute_commands(parts: &[&str], db: &Db) -> String {
    let command = parts[0];
    match command {
        // General
        "PING" => "PONG\n".to_string(),
        // Strings
        "SET" | "GET" | "DEL" | "EXISTS" | "TTL" => string::handle(parts, db).await,
        // Hash Map
        "HSET" | "HGET" | "HDEL" | "HTTL" => hashmap::handle(parts, db).await,
        _ => format!("Unknown command {}\n", parts[0]).to_string(),
    }
}
