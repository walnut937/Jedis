use crate::store::Db;
use crate::store::hashmap;

pub async fn handle(parts: &[&str], db: &Db) -> String {
    match parts[0] {
        "HSET" => {
            if parts.len() != 4 {
                return "ERR usage: HSET key field value\n".to_string();
            }
            let key = parts[1];
            let field = parts[2];
            let value = parts[3];
            match hashmap::hset(db, key, field, value).await {
                Ok(_) => "OK\n".to_string(),
                Err(e) => format!("ERR {}\n", e),
            }
        }

        "HGET" => {
            if parts.len() != 3 {
                return "ERR usage: HGET key field\n".to_string();
            }
            let key = parts[1];
            let field = parts[2];
            match hashmap::hget(db, key, field).await {
                Some(val) => format!("{}\n", val),
                None => "nil\n".to_string(),
            }
        }

        "HDEL" => {
            if parts.len() != 3 {
                return "ERR usage: HDEL key field\n".to_string();
            }
            let key = parts[1];
            let field = parts[2];
            match hashmap::hdel(db, key, field).await {
                true => "OK\n".to_string(),
                false => "ERR no such key/field\n".to_string(),
            }
        }

        _ => "UNKNOWN HASH COMMAND\n".to_string(),
    }
}
