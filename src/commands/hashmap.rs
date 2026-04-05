use crate::resp::writer;
use crate::store::hashmap;
use crate::store::{Db, Stats};

pub async fn handle(parts: &[&str], db: &Db, stats: &Stats) -> String {
    if parts.is_empty() {
        return writer::error("Empty command");
    }

    let command = parts[0].to_uppercase();
    match command.as_str() {
        "HSET" => match parts {
            [_, key, field, value] => match hashmap::hset(stats, db, key, field, value, None).await
            {
                Ok(_) => writer::simple_string("OK"),
                Err(e) => writer::error(&e),
            },
            [_, key, field, value, "EX", secs] => match secs.parse::<u64>() {
                Ok(n) => match hashmap::hset(stats, db, key, field, value, Some(n)).await {
                    Ok(_) => writer::simple_string("OK"),
                    Err(e) => writer::error(&e),
                },
                Err(_) => writer::error("Expiry must be a number"),
            },
            _ => writer::error("Usage: HSET key field value [EX seconds]"),
        },

        "HGET" => match parts {
            [_, key, field] => match hashmap::hget(db, key, field).await {
                Ok(Some(val)) => writer::bulk_string(&val),
                Ok(None) => writer::nil(),
                Err(e) => writer::error(&e),
            },
            _ => writer::error("Usage: HGET key field"),
        },

        "HDEL" => match parts {
            [_, key, field] => {
                if hashmap::hdel(stats, db, key, field).await {
                    writer::simple_string("OK")
                } else {
                    writer::error("No such key/field")
                }
            }
            _ => writer::error("Usage: HDEL key field"),
        },

        "HTTL" => match parts {
            [_, key] => match hashmap::httl(db, key).await {
                -2 => writer::nil(),
                -1 => writer::simple_string("no expiry"),
                n => writer::integer(n),
            },
            _ => writer::error("Usage: HTTL key"),
        },

        _ => writer::error("Unknown HASH command"),
    }
}
