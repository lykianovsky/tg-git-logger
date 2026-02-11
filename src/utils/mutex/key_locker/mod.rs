use std::{collections::HashMap, hash::Hash, sync::Arc};
use tokio::sync::{Mutex, OwnedMutexGuard};

pub struct KeyLocker<K>
where
    K: Eq + Hash + Clone,
{
    map: Mutex<HashMap<K, Arc<Mutex<()>>>>,
}

impl<K> KeyLocker<K>
where
    K: Eq + Hash + Clone,
{
    pub fn new() -> Self {
        Self {
            map: Mutex::new(HashMap::new()),
        }
    }

    pub async fn lock(&self, key: K) -> OwnedMutexGuard<()> {
        let key_mutex = {
            let mut map = self.map.lock().await;

            map.entry(key)
                .or_insert_with(|| Arc::new(Mutex::new(())))
                .clone()
        };

        key_mutex.lock_owned().await
    }
}
