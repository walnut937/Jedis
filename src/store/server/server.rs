use crate::store::{Db, RedisValue};

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
