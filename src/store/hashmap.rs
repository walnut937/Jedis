use crate::store::{Db, Entry, RedisValue, Stats, estimate_size};
use std::collections::HashMap;
use std::time::{Duration, Instant};

pub async fn hset(
    stats: &Stats,
    db: &Db,
    key: &str,
    field: &str,
    value: &str,
    ttl_secs: Option<u64>,
) -> Result<(), String> {
    let mut map = db.lock().await;
    if let Some(existing) = map.get(key) {
        let old_size = estimate_size(key, &existing.value);
        stats.sub_memory(old_size);
    }

    let entry = map.entry(key.to_string()).or_insert(Entry {
        value: RedisValue::Hash(HashMap::new()),
        expires_at: ttl_secs.map(|secs| Instant::now() + Duration::from_secs(secs)),
    });
    match &mut entry.value {
        RedisValue::Hash(hash) => {
            hash.insert(field.to_string(), value.to_string());
            let new_size = estimate_size(key, &entry.value);
            stats.add_memory(new_size);
            Ok(())
        }
        _ => Err("WRONGTYPE key holds a non-hash value".to_string()),
    }
}

pub async fn hget(db: &Db, key: &str, field: &str) -> Result<Option<String>, String> {
    let mut map = db.lock().await;
    match map.get(key) {
        Some(entry) if entry.is_expired() => {
            map.remove(key);
            Ok(None)
        }
        Some(entry) => match &entry.value {
            RedisValue::Hash(hash) => Ok(hash.get(field).cloned()),
            _ => Err("WRONGTYPE key holds a non-hash value".to_string()),
        },
        None => Ok(None),
    }
}

pub async fn hdel(stats: &Stats, db: &Db, key: &str, field: &str) -> bool {
    let mut map = db.lock().await;
    match map.get_mut(key) {
        Some(entry) if entry.is_expired() => {
            let size = estimate_size(key, &entry.value);
            stats.sub_memory(size);
            map.remove(key);
            false
        }
        Some(entry) => match &mut entry.value {
            RedisValue::Hash(hash) => {
                let removed = hash.remove(field).is_some();
                if removed {
                    let new_size = estimate_size(key, &entry.value);
                    stats.sub_memory(new_size);
                }
                removed
            }
            _ => false,
        },
        None => false,
    }
}

pub async fn httl(db: &Db, key: &str) -> i64 {
    let map = db.lock().await;
    match map.get(key) {
        None => -2,
        Some(entry) if entry.is_expired() => -2,
        Some(Entry {
            expires_at: None, ..
        }) => -1,
        Some(Entry {
            expires_at: Some(expiry),
            ..
        }) => expiry.duration_since(Instant::now()).as_secs() as i64,
    }
}
