use crate::config::SharedConfig;
use crate::resp::writer;
use crate::store::config_store::{config_get, config_rewrite, config_set};
use crate::store::server::{data_type, flush_db, get_db_size, get_keys, server_info};
use crate::store::{Db, Stats};

pub async fn handle(
    parts: &[&str],
    db: &Db,
    stats: &Stats,
    port: u16,
    config: &SharedConfig,
) -> String {
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

        "CONFIG" => match parts {
            [_, sub, key] if sub.eq_ignore_ascii_case("GET") => {
                let pairs: Vec<(String, String)> = config_get(config, key).await;

                if pairs.is_empty() {
                    writer::empty_array()
                } else {
                    let flat: Vec<String> =
                        pairs.into_iter().flat_map(|(k, v)| vec![k, v]).collect();

                    writer::array(&flat)
                }
            }

            [_, sub, key, value] if sub.eq_ignore_ascii_case("SET") => {
                match config_set(config, key, value).await {
                    Ok(_) => writer::simple_string("OK"),
                    Err(e) => writer::error(&e),
                }
            }

            [_, sub] if sub.eq_ignore_ascii_case("REWRITE") => match config_rewrite(config).await {
                Ok(_) => writer::simple_string("OK"),
                Err(e) => writer::error(&e),
            },

            _ => writer::error("usage: CONFIG GET key | CONFIG SET key value | CONFIG REWRITE"),
        },

        _ => writer::error("unknown server command"),
    }
}
