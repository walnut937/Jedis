use crate::store::Db;
use crate::store::string::{del, exists, get, set, ttl};

pub async fn handle(parts: &[&str], db: &Db) -> String {
    match parts[0] {
        "SET" => match parts {
            [_, key, value] => {
                set(db, key, value, None).await;
                "OK\n".to_string()
            }
            [_, key, value, "EX", secs] => match secs.parse::<u64>() {
                Ok(n) => {
                    set(db, key, value, Some(n)).await;
                    "OK\n".to_string()
                }
                Err(_) => "Err expiry seconds must be a number\n".to_string(),
            },
            _ => "Err usage: SET key value [EX seconds]\n".to_string(),
        },
        "GET" => match parts {
            [_, key] => match get(db, key).await {
                Ok(Some(val)) => format!("{}\n", val),
                Ok(None) => "nil\n".to_string(),
                Err(e) => format!("Err {}\n", e),
            },
            _ => "Err usage: GET key\n".to_string(),
        },
        "DEL" => match parts {
            [_, key] => match del(db, key).await {
                Some(_) => "OK\n".to_string(),
                None => "Err no such key found\n".to_string(),
            },
            _ => "Err usage: DEL key\n".to_string(),
        },
        "EXISTS" => match parts {
            [_, key] => {
                if exists(db, key).await {
                    "1\n".to_string()
                } else {
                    "0\n".to_string()
                }
            }
            _ => "Err usage: EXISTS key\n".to_string(),
        },
        "TTL" => match parts {
            [_, key] => match ttl(db, key).await {
                -2 => "nil\n".to_string(),
                -1 => "no expiry\n".to_string(),
                n => format!("{}s remaining\n", n),
            },
            _ => "Err usage: TTL key\n".to_string(),
        },
        _ => "UNKNOWN STRING COMMAND\n".to_string(),
    }
}
