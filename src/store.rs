use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

pub type Db = Arc<Mutex<HashMap<String, String>>>;

pub fn create_db() -> Db {
    Arc::new(Mutex::new(HashMap::new()))
}

pub async fn store_values(db: &Db, key: &str, value: &str) {
    let mut map = db.lock().await;
    map.insert(key.to_string(), value.to_string());
}

pub async fn get_values(db: &Db, key: &str) -> Option<String> {
    let map = db.lock().await;
    map.get(key).cloned()
}

pub async fn delete_values(db: &Db, key: &str) -> Option<String> {
    let mut map = db.lock().await;
    map.remove(key)
}

pub async fn exist_key(db: &Db, key: &str) -> bool {
    let map = db.lock().await;
    map.contains_key(key)
}
