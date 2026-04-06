use crate::config::{SharedConfig, validate_loglevel};

pub async fn config_get(config: &SharedConfig, key: &str) -> Vec<(String, String)> {
    let cfg = config.read().await;
    match key {
        "host" => vec![("host".to_string(), cfg.host.clone())],
        "port" => vec![("port".to_string(), cfg.port.to_string())],
        "password" => vec![(
            "password".to_string(),
            cfg.password.clone().unwrap_or_default(),
        )],
        "maxconnections" => vec![(
            "maxconnections".to_string(),
            cfg.max_connections.to_string(),
        )],
        "loglevel" => vec![("loglevel".to_string(), cfg.loglevel.clone())],
        "monitor_buffer" => vec![("monitor_buffer".to_string(), cfg.monitor_buffer.to_string())],
        "*" => vec![
            ("host".to_string(), cfg.host.clone()),
            ("port".to_string(), cfg.port.to_string()),
            (
                "password".to_string(),
                cfg.password.clone().unwrap_or_default(),
            ),
            (
                "maxconnections".to_string(),
                cfg.max_connections.to_string(),
            ),
            ("loglevel".to_string(), cfg.loglevel.clone()),
            ("monitor_buffer".to_string(), cfg.monitor_buffer.to_string()),
        ],
        _ => vec![],
    }
}

pub async fn config_set(config: &SharedConfig, key: &str, value: &str) -> Result<(), String> {
    // validate BEFORE acquiring write lock
    validate_config_set(key, value)?;

    let mut cfg = config.write().await;

    match key {
        "password" => {
            cfg.password = if value.is_empty() {
                None
            } else {
                Some(value.to_string())
            };
        }
        "maxconnections" => {
            cfg.max_connections = value.parse().unwrap(); // safe — validated above
        }
        "loglevel" => {
            cfg.loglevel = value.to_string(); // safe — validated above
        }
        "monitor_buffer" => {
            cfg.monitor_buffer = value.parse().unwrap(); // safe — validated above
        }
        _ => unreachable!(), // validate_config_set catches everything else
    }

    Ok(())
}

pub async fn config_rewrite(config: &SharedConfig) -> Result<(), String> {
    let cfg = config.read().await;
    cfg.save_to_conf()
}

// validate separately so we don't hold the write lock during validation
fn validate_config_set(key: &str, value: &str) -> Result<(), String> {
    match key {
        "port" | "host" => {
            Err("cannot change port or host at runtime — they are bound at startup. Restart with new --port or --host".to_string())
        }
        "password" => Ok(()),  // any string is valid
        "maxconnections" => {
            let n: usize = value.parse()
                .map_err(|_| format!("invalid maxconnections '{}' — must be a positive integer", value))?;
            if n == 0 {
                return Err("maxconnections must be greater than 0".to_string());
            }
            if n > 100_000 {
                return Err("maxconnections cannot exceed 100000".to_string());
            }
            Ok(())
        }
        "loglevel" => validate_loglevel(value),
        "monitor_buffer" => {
            let n: usize = value.parse()
                .map_err(|_| format!("invalid monitor_buffer '{}' — must be a positive integer", value))?;
            if n == 0 {
                return Err("monitor_buffer must be greater than 0".to_string());
            }
            if n > 10_000 {
                return Err("monitor_buffer cannot exceed 10000".to_string());
            }
            Ok(())
        }
        _ => Err(format!("unknown config key '{}' — valid keys: password, maxconnections, loglevel, monitor_buffer", key)),
    }
}
