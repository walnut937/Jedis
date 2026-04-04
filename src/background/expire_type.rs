use crate::store::Db;
use rand::seq::IteratorRandom;
use std::time::Duration;

pub async fn active_expiry(db: Db) {
    loop {
        tokio::time::sleep(Duration::from_millis(100)).await;

        loop {
            let mut map = db.lock().await;
            let total = map.len();

            if total == 0 {
                break;
            }

            // pick 20 random keys
            let sample_size = 20.min(total);
            let sampled_keys: Vec<String> = map
                .keys()
                .choose_multiple(&mut rand::thread_rng(), sample_size)
                .into_iter()
                .cloned()
                .collect();

            // check how many are expired
            let expired_keys: Vec<String> = sampled_keys
                .into_iter()
                .filter(|k| map.get(k).map_or(false, |e| e.is_expired()))
                .collect();

            let expired_count = expired_keys.len();

            // delete them
            for key in expired_keys {
                map.remove(&key);
            }

            // if more than 25% were expired → run again immediately
            // else → break and wait for next 100ms cycle
            if expired_count * 100 / sample_size < 25 {
                break;
            }

            // drop lock before looping again so clients aren't blocked
            drop(map);
        }
    }
}
