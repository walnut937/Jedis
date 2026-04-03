use crate::store::Db;
use crate::store::{get_values, store_values};

pub async fn execute_commands(parts: &[&str], db: &Db) -> String {
    let command = parts[0];

    match command {
        "PING" => "PONG\n".to_string(),
        "SET" => {
            if parts.len() != 3 {
                return "ERR usage: SET key value\n".to_string();
            }
            let key = parts[1];
            let value = parts[2];
            store_values(db, key, value).await;
            "OK\n".to_string()
        }
        "GET" => {
            if parts.len() != 2 {
                return "ERR usage: GET value\n".to_string();
            }
            let key = parts[1];
            let value = get_values(db, key).await;
            match value {
                Some(val) => format!("{}\n", val),
                None => "No value exist\n".to_string(),
            }
        }
        _ => "UNKNOWN\n".to_string(),
    }
}
