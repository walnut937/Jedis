use crate::store::{Db, Entry, RedisValue};
use std::time::{Duration, Instant};

pub async fn set(db: &Db, key: &str, value: &str, ttl_secs: Option<u64>) {
    let mut map = db.lock().await;
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

pub async fn del(db: &Db, key: &str) -> Option<()> {
    let mut map = db.lock().await;
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
