use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

pub enum RedisValue {
    String(String),
    Hash(HashMap<String, String>),
    _List(Vec<String>),
}

pub struct Entry {
    pub value: RedisValue,
}

pub type Db = Arc<Mutex<HashMap<String, Entry>>>;

pub fn create_db() -> Db {
    Arc::new(Mutex::new(HashMap::new()))
}
