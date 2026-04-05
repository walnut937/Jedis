use crate::store::hashmap;
use crate::store::{Db, Stats};

pub async fn handle(parts: &[&str], db: &Db, stats: &Stats) -> String {
    let command = parts[0].to_uppercase();
    match command.as_str() {
        "HSET" => match parts {
            [_, key, field, value] => match hashmap::hset(stats, db, key, field, value, None).await
            {
                Ok(_) => "OK\n".to_string(),
                Err(e) => format!("ERR {}\n", e),
            },
            [_, key, field, value, "EX", secs] => match secs.parse::<u64>() {
                Ok(n) => match hashmap::hset(stats, db, key, field, value, Some(n)).await {
                    Ok(_) => "OK\n".to_string(),
                    Err(e) => format!("ERR {}\n", e),
                },
                Err(_) => "ERR expiry must be a number\n".to_string(),
            },
            _ => "ERR usage: HSET key field value [EX seconds]\n".to_string(),
        },

        "HGET" => match parts {
            [_, key, field] => match hashmap::hget(db, key, field).await {
                Ok(Some(val)) => format!("{}\n", val),
                Ok(None) => "nil\n".to_string(),
                Err(e) => format!("ERR {}\n", e),
            },
            _ => "ERR usage: HGET key field\n".to_string(),
        },

        "HDEL" => match parts {
            [_, key, field] => match hashmap::hdel(stats, db, key, field).await {
                true => "OK\n".to_string(),
                false => "ERR no such key/field\n".to_string(),
            },
            _ => "ERR usage: HDEL key field\n".to_string(),
        },

        "HTTL" => match parts {
            [_, key] => match hashmap::httl(db, key).await {
                -2 => "nil\n".to_string(),
                -1 => "no expiry\n".to_string(),
                n => format!("{}s remaining\n", n),
            },
            _ => "ERR usage: HTTL key\n".to_string(),
        },

        _ => "UNKNOWN HASH COMMAND\n".to_string(),
    }
}
