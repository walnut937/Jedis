use crate::store::{Db, Stats, estimate_size};
use rand::seq::IteratorRandom;
use std::time::Duration;

pub async fn active_expiry(db: Db, stats: Stats) {
    loop {
        tokio::time::sleep(Duration::from_millis(100)).await;
        loop {
            let mut map = db.lock().await;
            let total = map.len();
            if total == 0 {
                break;
            }

            let sample_size = 20.min(total);
            let mut rng = rand::thread_rng();

            let sampled_keys: Vec<String> = map
                .keys()
                .choose_multiple(&mut rng, sample_size)
                .into_iter()
                .cloned()
                .collect();

            let expired_keys: Vec<String> = sampled_keys
                .into_iter()
                .filter(|k| map.get(k).map_or(false, |e| e.is_expired()))
                .collect();

            let expired_count = expired_keys.len();

            for key in &expired_keys {
                if let Some(entry) = map.get(key) {
                    let size = estimate_size(key, &entry.value);
                    stats.sub_memory(size); // subtract memory
                }
                map.remove(key);
            }

            if expired_count * 100 / sample_size < 25 {
                break;
            }
            drop(map);
        }
    }
}
