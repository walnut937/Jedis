use crate::resp::writer;
use crate::store::string::{del, exists, get, increment, set, ttl};
use crate::store::{Db, Stats};

pub async fn handle(parts: &[&str], db: &Db, stats: &Stats) -> String {
    if parts.is_empty() {
        return writer::error("Empty command");
    }

    let command = parts[0].to_uppercase();
    match command.as_str() {
        "SET" => match parts {
            [_, key, value] => {
                set(stats, db, key, value, None).await;
                writer::simple_string("OK")
            }
            [_, key, value, "EX", secs] => match secs.parse::<u64>() {
                Ok(n) => {
                    set(stats, db, key, value, Some(n)).await;
                    writer::simple_string("OK")
                }
                Err(_) => writer::error("Expiry seconds must be a number"),
            },
            _ => writer::error("Usage: SET key value [EX seconds]"),
        },

        "GET" => match parts {
            [_, key] => match get(db, key).await {
                Ok(Some(val)) => writer::bulk_string(&val),
                Ok(None) => writer::nil(),
                Err(e) => writer::error(&e),
            },
            _ => writer::error("Usage: GET key"),
        },

        "DEL" => match parts {
            [_, key] => match del(stats, db, key).await {
                Some(_) => writer::simple_string("OK"),
                None => writer::error("No such key"),
            },
            _ => writer::error("Usage: DEL key"),
        },

        "EXISTS" => match parts {
            [_, key] => {
                if exists(db, key).await {
                    writer::simple_string("1")
                } else {
                    writer::simple_string("0")
                }
            }
            _ => writer::error("Usage: EXISTS key"),
        },

        "TTL" => match parts {
            [_, key] => match ttl(db, key).await {
                -2 => writer::nil(),
                -1 => writer::simple_string("no expiry"),
                n => writer::integer(n),
            },
            _ => writer::error("Usage: TTL key"),
        },

        "INCR" => match parts {
            [_, key] => increment(stats, db, key, 1).await,
            _ => writer::error("Usage: INCR key"),
        },

        "DECR" => match parts {
            [_, key] => increment(stats, db, key, -1).await,
            _ => writer::error("Usage: DECR key"),
        },

        "INCRBY" => match parts {
            [_, key, amount] => match amount.parse::<i64>() {
                Ok(n) => increment(stats, db, key, n).await,
                Err(_) => writer::error("Amount must be a number"),
            },
            _ => writer::error("Usage: INCRBY key amount"),
        },

        "DECRBY" => match parts {
            [_, key, amount] => match amount.parse::<i64>() {
                Ok(n) => increment(stats, db, key, -n).await,
                Err(_) => writer::error("Amount must be a number"),
            },
            _ => writer::error("Usage: DECRBY key amount"),
        },

        _ => writer::error("Unknown STRING command"),
    }
}
