use crate::store::Db;
use crate::store::string::{del, exists, get, set};

pub async fn handle(parts: &[&str], db: &Db) -> String {
    match parts[0] {
        "SET" => {
            if parts.len() != 3 {
                return "ERR usage: SET key value\n".to_string();
            }
            let key = parts[1];
            let value = parts[2];
            set(db, key, value).await;
            "OK\n".to_string()
        }
        "GET" => {
            if parts.len() != 2 {
                return "ERR usage: GET value\n".to_string();
            }
            let key = parts[1];
            match get(db, key).await {
                Ok(Some(val)) => format!("{}\n", val),
                Ok(None) => "nil\n".to_string(),
                Err(e) => format!("Err {}\n", e),
            }
        }
        "DEL" => {
            if parts.len() != 2 {
                return "ERR usage: No value to delete\n".to_string();
            }
            let key = parts[1];
            let result = del(db, key).await;
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
            let exists = exists(db, key).await;
            if exists {
                "1\n".to_string()
            } else {
                "0\n".to_string()
            }
        }

        _ => "UNKNOWN STRING COMMAND\n".to_string(),
    }
}
