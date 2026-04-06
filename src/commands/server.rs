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

        "CONFIG" => {
            // normalize sub-commands too
            if parts.len() < 2 {
                return writer::error(
                    "usage: CONFIG GET key | CONFIG SET key value | CONFIG REWRITE",
                );
            }

            let sub = parts[1].to_uppercase();

            match sub.as_str() {
                "GET" => {
                    if parts.len() != 3 {
                        return writer::error("usage: CONFIG GET key");
                    }
                    let pairs = config_get(config, parts[2]).await;
                    if pairs.is_empty() {
                        writer::empty_array()
                    } else {
                        let flat: Vec<String> =
                            pairs.into_iter().flat_map(|(k, v)| vec![k, v]).collect();
                        writer::array(&flat)
                    }
                }

                "SET" => {
                    if parts.len() < 4 {
                        return writer::error("usage: CONFIG SET key value");
                    }
                    let key = parts[2].to_lowercase();
                    let value = parts[3];

                    if key == "password" {
                        match config_set(config, "password", value).await {
                            Ok(_) => writer::simple_string("OK password set and auth enabled"),
                            Err(e) => writer::error(&e),
                        }
                    } else {
                        match config_set(config, &key, value).await {
                            Ok(_) => writer::simple_string("OK"),
                            Err(e) => writer::error(&e),
                        }
                    }
                }

                "UNSET" => {
                    if parts.len() < 3 {
                        return writer::error("usage: CONFIG UNSET password <current>");
                    }
                    let key = parts[2].to_lowercase();

                    if key != "password" {
                        return writer::error("CONFIG UNSET only supports password");
                    }

                    let cfg = config.read().await;

                    if !cfg.auth_enabled {
                        return writer::simple_string("OK auth already disabled");
                    }

                    match &cfg.password {
                        None => {
                            drop(cfg);
                            match config_set(config, "password", "").await {
                                Ok(_) => writer::simple_string("OK auth disabled"),
                                Err(e) => writer::error(&e),
                            }
                        }
                        Some(expected) => {
                            if parts.len() < 4 {
                                return writer::error(
                                    "provide current password: CONFIG UNSET password <current>",
                                );
                            }
                            if parts[3] != expected {
                                writer::error("WRONGPASS cannot remove password — wrong password")
                            } else {
                                drop(cfg);
                                match config_set(config, "password", "").await {
                                    Ok(_) => writer::simple_string(
                                        "OK password removed and auth disabled",
                                    ),
                                    Err(e) => writer::error(&e),
                                }
                            }
                        }
                    }
                }

                "REWRITE" => match config_rewrite(config).await {
                    Ok(_) => writer::simple_string("OK saved to jedis.conf"),
                    Err(e) => writer::error(&e),
                },

                _ => writer::error(
                    "usage: CONFIG GET key | CONFIG SET key value | CONFIG UNSET password <current> | CONFIG REWRITE",
                ),
            }
        }

        _ => writer::error("unknown server command"),
    }
}
