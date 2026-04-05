use crate::store::{Db, Entry, RedisValue, Stats, estimate_size};
use std::time::{Duration, Instant};

pub async fn set(stats: &Stats, db: &Db, key: &str, value: &str, ttl_secs: Option<u64>) {
    let mut map = db.lock().await;
    if let Some(existing) = map.get(key) {
        let old_size = estimate_size(key, &existing.value);
        stats.sub_memory(old_size);
    }

    let new_value = RedisValue::String(value.to_string());
    let new_size = estimate_size(key, &new_value);
    stats.add_memory(new_size);

    map.insert(
        key.to_string(),
        Entry {
            value: RedisValue::String(value.to_string()),
            expires_at: ttl_secs.map(|secs| Instant::now() + Duration::from_secs(secs)),
        },
    );
}

pub async fn get(db: &Db, key: &str) -> Result<Option<String>, String> {
    let mut map = db.lock().await;
    match map.get(key) {
        Some(entry) if entry.is_expired() => {
            map.remove(key);
            Ok(None)
        }
        Some(entry) => match &entry.value {
            RedisValue::String(s) => Ok(Some(s.clone())),
            _ => Err("WRONGTYPE key holds a non-string value".to_string()),
        },
        None => Ok(None),
    }
}

pub async fn del(stats: &Stats, db: &Db, key: &str) -> Option<()> {
    let mut map = db.lock().await;
    if let Some(existing) = map.get(key) {
        let size = estimate_size(key, &existing.value);
        stats.sub_memory(size);
    }
    map.remove(key).map(|_| ())
}

pub async fn exists(db: &Db, key: &str) -> bool {
    let map = db.lock().await;
    map.contains_key(key)
}

pub async fn ttl(db: &Db, key: &str) -> i64 {
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

pub async fn increment(stats: &Stats, db: &Db, key: &str, by: i64) -> String {
    let mut map = db.lock().await;

    let current = match map.get(key) {
        Some(entry) => match &entry.value {
            RedisValue::String(value) => match value.parse::<i64>() {
                Ok(n) => n,
                Err(_) => return "ERR value is not an integer\n".to_string(),
            },
            _ => return "ERR WRONGTYPE key holds a non-string value\n".to_string(),
        },
        None => 0,
    };

    // subtract old memory if key existed
    if let Some(existing) = map.get(key) {
        let old_size = estimate_size(key, &existing.value);
        stats.sub_memory(old_size);
    }

    let new_value = current + by;
    let new_redis_value = RedisValue::String(new_value.to_string());

    // add new memory
    let new_size = estimate_size(key, &new_redis_value);
    stats.add_memory(new_size);

    map.insert(
        key.to_string(),
        Entry {
            value: new_redis_value,
            expires_at: None,
        },
    );

    format!("{}\n", new_value)
}
