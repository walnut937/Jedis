use crate::store::{Db, RedisValue, Stats};

pub async fn get_db_size(db: &Db) -> String {
    let map = db.lock().await;
    map.len().to_string()
}

pub async fn flush_db(db: &Db) -> String {
    let mut map = db.lock().await;
    map.clear();
    "OK\n".to_string()
}

pub async fn data_type(db: &Db, data: &str) -> String {
    let map = db.lock().await;

    match map.get(data) {
        None => "None\n".to_string(),
        Some(entry) => match &entry.value {
            RedisValue::String(_) => "String\n".to_string(),
            RedisValue::Hash(_) => "Hash\n".to_string(),
        },
    }
}

pub async fn get_keys(db: &Db, pattern: &str) -> String {
    let map = db.lock().await;
    let keys: Vec<&String> = map
        .keys()
        .filter(|k| {
            if pattern == "*" {
                true // * means all keys
            } else {
                k.starts_with(pattern.trim_end_matches('*'))
            }
        })
        .collect();

    if keys.is_empty() {
        "empty\n".to_string()
    } else {
        keys.iter()
            .enumerate()
            .map(|(i, k)| format!("{}) {}\n", i + 1, k))
            .collect()
    }
}

fn humanize(bytes: usize) -> String {
    if bytes < 1024 {
        format!("{}B", bytes)
    } else if bytes < 1024 * 1024 {
        format!("{:.2}KB", bytes as f64 / 1024.0)
    } else if bytes < 1024 * 1024 * 1024 {
        format!("{:.2}MB", bytes as f64 / (1024.0 * 1024.0))
    } else {
        format!("{:.2}G", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
    }
}

pub async fn server_info(db: &Db, stats: &Stats, port: u16) -> String {
    let map = db.lock().await;

    let uptime = stats.started_at.elapsed().as_secs();
    let uptime_days = uptime / 86400;

    let total_keys = map.len();

    let commands = stats
        .total_commands
        .load(std::sync::atomic::Ordering::Relaxed);

    let total_connections = stats
        .total_connections
        .load(std::sync::atomic::Ordering::Relaxed);

    let connected_clients = stats
        .connected_clients
        .load(std::sync::atomic::Ordering::Relaxed);

    let used_memory = stats.used_memory.load(std::sync::atomic::Ordering::Relaxed);

    let used_memory_human = humanize(used_memory);

    format!(
        "# Server\n\
        jedis_version:0.1.0\n\
        uptime_in_seconds:{}\n\
        uptime_in_days:{}\n\
        tcp_port:{}\n\
        \n\
        # Clients\n\
        connected_clients:{}\n\
        \n\
        # Memory\n\
        used_memory:{}\n\
        used_memory_human:{}\n\
        \n\
        # Stats\n\
        total_keys:{}\n\
        total_commands_processed:{}\n\
        total_connections_received:{}\n",
        uptime,
        uptime_days,
        port,
        connected_clients,
        used_memory,
        used_memory_human,
        total_keys,
        commands,
        total_connections
    )
}
