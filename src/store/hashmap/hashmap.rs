use crate::store::{Db, Entry, RedisValue};
use std::collections::HashMap;

pub async fn hset(db: &Db, key: &str, field: &str, value: &str) -> Result<(), String> {
    let mut map = db.lock().await;

    let entry = map.entry(key.to_string()).or_insert(Entry {
        value: RedisValue::Hash(HashMap::new()),
    });

    match &mut entry.value {
        RedisValue::Hash(hash) => {
            hash.insert(field.to_string(), value.to_string());
            Ok(())
        }
        _ => Err("WRONGTYPE key holds a non-hash value".to_string()),
    }
}
pub async fn hget(db: &Db, key: &str, field: &str) -> Option<String> {
    let map = db.lock().await;
    match map.get(key) {
        Some(entry) => match &entry.value {
            RedisValue::Hash(hash) => hash.get(field).cloned(),
            _ => None, // wrong type, return None
        },
        None => None,
    }
}
pub async fn hdel(db: &Db, key: &str, field: &str) -> bool {
    let mut map = db.lock().await;
    match map.get_mut(key) {
        Some(entry) => match &mut entry.value {
            RedisValue::Hash(hash) => hash.remove(field).is_some(),
            _ => false,
        },
        None => false,
    }
}
