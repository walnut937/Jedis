use crate::store::{Db, RedisValue, Stats};
use std::sync::atomic::Ordering;

pub async fn get_db_size(db: &Db) -> usize {
    let map = db.lock().await;
    map.len()
}

pub async fn flush_db(db: &Db) {
    let mut map = db.lock().await;
    map.clear();
}

pub async fn data_type(db: &Db, key: &str) -> String {
    let map = db.lock().await;
    match map.get(key) {
        None => "none".to_string(),
        Some(entry) => match &entry.value {
            RedisValue::String(_) => "string".to_string(),
            RedisValue::Hash(_) => "hash".to_string(),
        },
    }
}

pub async fn get_keys(db: &Db, pattern: &str) -> Vec<String> {
    let map = db.lock().await;
    map.keys()
        .filter(|k| {
            if pattern == "*" {
                true
            } else {
                k.starts_with(pattern.trim_end_matches('*'))
            }
        })
        .cloned()
        .collect()
}

fn humanize(bytes: usize) -> String {
    if bytes < 1024 {
        format!("{}B", bytes)
    } else if bytes < 1024 * 1024 {
        format!("{:.2}KB", bytes as f64 / 1024.0)
    } else if bytes < 1024 * 1024 * 1024 {
        format!("{:.2}MB", bytes as f64 / (1024.0 * 1024.0))
    } else {
        format!("{:.2}GB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
    }
}

pub async fn server_info(db: &Db, stats: &Stats, port: u16) -> String {
    let map = db.lock().await;
    let uptime = stats.started_at.elapsed().as_secs();
    let uptime_days = uptime / 86400;
    let total_keys = map.len();
    let commands = stats.total_commands.load(Ordering::Relaxed);
    let total_connections = stats.total_connections.load(Ordering::Relaxed);
    let connected_clients = stats.connected_clients.load(Ordering::Relaxed);
    let used_memory = stats.used_memory.load(Ordering::Relaxed);
    let used_memory_human = humanize(used_memory);

    format!(
        "# Server\r\njedis_version:0.1.0\r\nuptime_in_seconds:{}\r\nuptime_in_days:{}\r\ntcp_port:{}\r\n\r\n# Clients\r\nconnected_clients:{}\r\n\r\n# Memory\r\nused_memory:{}\r\nused_memory_human:{}\r\n\r\n# Stats\r\ntotal_keys:{}\r\ntotal_commands_processed:{}\r\ntotal_connections_received:{}\r\n",
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
