use crate::resp::writer;
use crate::store::server::{data_type, flush_db, get_db_size, get_keys, server_info};
use crate::store::{Db, Stats};

pub async fn handle(parts: &[&str], db: &Db, stats: &Stats, port: u16) -> String {
    match parts[0].to_uppercase().as_str() {
        "PING" => writer::simple_string("PONG"),

        "ECHO" => {
            if parts.len() != 2 {
                return writer::error("usage: ECHO message");
            }
            writer::bulk_string(parts[1])
        }

        "DBSIZE" => writer::integer(get_db_size(db).await as i64),

        "FLUSHDB" => {
            flush_db(db).await;
            writer::simple_string("OK")
        }

        "TYPE" => {
            if parts.len() != 2 {
                return writer::error("usage: TYPE key");
            }
            writer::simple_string(&data_type(db, parts[1]).await)
        }

        "KEYS" => {
            if parts.len() != 2 {
                return writer::error("usage: KEYS pattern");
            }
            writer::array(&get_keys(db, parts[1]).await)
        }

        "INFO" => {
            let info_str = server_info(db, stats, port).await;
            let lines: Vec<String> = info_str.lines().map(|l| l.to_string()).collect();
            writer::array(&lines)
        }

        _ => writer::error("unknown server command"),
    }
}
