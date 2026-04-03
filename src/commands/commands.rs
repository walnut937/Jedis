use crate::store::Db;
use crate::store::{delete_values, exist_key, get_values, store_values};

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
        "DEL" => {
            if parts.len() != 2 {
                return "ERR usage: No value to delete\n".to_string();
            }
            let key = parts[1];
            let result = delete_values(db, key).await;
            match result {
                Some(_) => "OK\n".to_string(),
                None => "Err no such key found\n".to_string(),
            }
        }
        "EXISTS" => {
            if parts.len() != 2 {
                return "Err usage: No exist key\n".to_string();
            }
            let key = parts[1];
            let exists = exist_key(db, key).await;
            if exists {
                "1\n".to_string()
            } else {
                "0\n".to_string()
            }
        }
        _ => format!("unknown command '{}'\n", parts[0]).to_string(),
    }
}
