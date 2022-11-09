use std::{collections::HashMap, sync::{Arc, Once}};

use tokio::sync::Mutex;

use crate::resp::command::SetCommandFlags;

/**
 * Storage implementation for Redis
 * In-memory implementation
 */
pub struct RedisStore {
    store: HashMap<String, String>
}

type SharedRedisStore = Arc<Mutex<RedisStore>>;
static mut SHARED_STORE: Option<SharedRedisStore> = None;
static STORE_INIT: Once = Once::new();

impl RedisStore {
    pub fn init() {
        STORE_INIT.call_once(|| unsafe {
            // This is safe because static store can only initialise once from this method only
            SHARED_STORE = Some(Arc::new(Mutex::new(RedisStore::default())));
        })
    }

    fn default() -> Self {
        Self { store: HashMap::new() }
    }
    
    pub fn get_shared_store<'a>() -> SharedRedisStore {
        if !(STORE_INIT.is_completed()) {
            Self::init()
        }

        unsafe {
            // This is safe because static store is protected behind a thread-safe reference
            return Arc::clone(&SHARED_STORE.as_mut().unwrap());
        }
    }
    
    pub fn get(&self, key: String) -> Option<String> {
        println!("Get: {}", key);
        self.store.get(&key).map(|s| s.to_owned())
    }

    pub fn set(&mut self, key: String, value: String, _flags: SetCommandFlags) {
        println!("Set: {}, {}", key, value);
        self.store.insert(key, value);
    }
}
