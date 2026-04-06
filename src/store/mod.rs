use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::time::Instant;
use tokio::sync::Mutex;

pub mod config_store;
pub mod hashmap;
pub mod server;
pub mod string;

pub enum RedisValue {
    String(String),
    Hash(HashMap<String, String>),
    // _List(Vec<String>),
}

pub struct ServerStats {
    pub started_at: Instant,
    pub total_commands: AtomicU64,
    pub total_connections: AtomicU64,
    pub used_memory: AtomicUsize,
    pub connected_clients: AtomicUsize,
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

impl ServerStats {
    fn new() -> ServerStats {
        ServerStats {
            started_at: Instant::now(),
            total_commands: AtomicU64::new(0),
            total_connections: AtomicU64::new(0),
            connected_clients: AtomicUsize::new(0),
            used_memory: AtomicUsize::new(0),
        }
    }

    pub fn increment_connections(&self) {
        self.connected_clients.fetch_add(1, Ordering::Relaxed);
        self.total_connections.fetch_add(1, Ordering::Relaxed);
    }

    pub fn increment_commands(&self) {
        self.total_commands.fetch_add(1, Ordering::Relaxed);
    }

    pub fn decrement_connections(&self) {
        self.connected_clients
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |val| {
                Some(val.saturating_sub(1))
            })
            .ok();
    }

    pub fn add_memory(&self, bytes: usize) {
        self.used_memory.fetch_add(bytes, Ordering::Relaxed);
    }
    pub fn sub_memory(&self, bytes: usize) {
        self.used_memory.fetch_sub(bytes, Ordering::Relaxed);
    }
}

pub fn estimate_size(key: &str, value: &RedisValue) -> usize {
    let key_size = key.len();
    let value_size = match value {
        RedisValue::String(s) => s.len(),
        RedisValue::Hash(h) => h.iter().map(|(k, v)| k.len() + v.len()).sum(),
    };
    key_size + value_size + 64
}

pub type Db = Arc<Mutex<HashMap<String, Entry>>>;
pub type Stats = Arc<ServerStats>;

pub fn create_db() -> Db {
    Arc::new(Mutex::new(HashMap::new()))
}

pub fn create_stats() -> Stats {
    Arc::new(ServerStats::new())
}
