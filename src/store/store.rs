use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::Mutex;

pub enum RedisValue {
    String(String),
    Hash(HashMap<String, String>),
    // _List(Vec<String>),
}

pub struct Entry {
    pub value: RedisValue,
    pub expires_at: Option<Instant>,
}

impl Entry {
    pub fn is_expired(&self) -> bool {
        match self.expires_at {
            Some(expiry) => Instant::now() > expiry,
            None => false,
        }
    }
}

pub type Db = Arc<Mutex<HashMap<String, Entry>>>;

pub fn create_db() -> Db {
    Arc::new(Mutex::new(HashMap::new()))
}
