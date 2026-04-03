use crate::store::{Db, Entry, RedisValue};

pub async fn set(db: &Db, key: &str, value: &str) {
    let mut map = db.lock().await;
    map.insert(
        key.to_string(),
        Entry {
            value: RedisValue::String(value.to_string()),
        },
    );
}

pub async fn get(db: &Db, key: &str) -> Result<Option<String>, String> {
    let map = db.lock().await;
    match map.get(key) {
        Some(entry) => match &entry.value {
            RedisValue::String(s) => Ok(Some(s.clone())),
            _ => Err("Wrongtype key holds a non-string value".to_string()),
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
