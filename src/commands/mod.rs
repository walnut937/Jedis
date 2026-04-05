use crate::store::{Db, Stats};

pub mod hashmap;
pub mod server;
pub mod string;

pub async fn execute_commands(parts: &[&str], db: &Db, stats: &Stats, port: u16) -> String {
    let command = parts[0].to_uppercase();
    stats.increment_commands();
    match command.as_str() {
        // Server
        "PING" | "ECHO" | "DBSIZE" | "INFO" | "FLUSHDB" | "TYPE" | "KEYS" => {
            server::handle(parts, &db, &stats, port).await
        }
        // Strings
        "SET" | "GET" | "DEL" | "EXISTS" | "TTL" | "INCR" | "DECR" | "INCRBY" | "DECRBY" => {
            string::handle(parts, &db, &stats).await
        }
        // Hash Map
        "HSET" | "HGET" | "HDEL" | "HTTL" => hashmap::handle(parts, &db, &stats).await,
        _ => format!("Unknown command {}\n", parts[0]).to_string(),
    }
}
