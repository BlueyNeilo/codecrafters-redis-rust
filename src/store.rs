use std::collections::HashMap;

/**
 * Storage implementation for Redis
 * In-memory implementation
 */
pub struct RedisStore {
    store: HashMap<String, String>
}

impl RedisStore {
    pub fn new() -> Self {
        Self { store: HashMap::new() }
    }
    
    pub fn get(&self, key: String) -> Option<String> {
        println!("Get: {}", key);
        self.store.get(&key).map(|s| s.to_owned())
    }

    pub fn set(&mut self, key: String, value: String) {
        println!("Set: {}, {}", key, value);
        self.store.insert(key, value);
    }
}
