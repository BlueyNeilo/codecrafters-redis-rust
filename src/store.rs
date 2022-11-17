use std::{collections::HashMap, sync::{Arc, Once}, time::UNIX_EPOCH};

use tokio::sync::Mutex;

use crate::{resp::command::{SetCommandFlags, SetCommandExistFlag, SetCommandTTLFlag}, clock::Clock};

type EpochMillisecond = u64;

/**
 * Storage implementation for Redis
 * In-memory implementation
 */
pub struct RedisStore {
    store: HashMap<String, String>,
    ttl_store: HashMap<String, EpochMillisecond>,
}

type SharedRedisStore = Arc<Mutex<RedisStore>>;
static mut SHARED_STORE: Option<SharedRedisStore> = None;
static STORE_INIT: Once = Once::new();

impl RedisStore {
    pub fn init() {
        STORE_INIT.call_once(|| unsafe {
            // This is safe because static store can only initialise/modify once from this method only
            SHARED_STORE = Some(Arc::new(Mutex::new(RedisStore::default())));
        })
    }

    fn default() -> Self {
        Self { store: HashMap::new(), ttl_store: HashMap::new() }
    }
    
    pub fn get_shared_store<'a>() -> SharedRedisStore {
        if !(STORE_INIT.is_completed()) {
            Self::init()
        }

        unsafe {
            // This is safe because static store is protected behind a thread-safe reference
            // It can not give any references to shared store until it is initialised
            return Arc::clone(&SHARED_STORE.as_mut().unwrap());
        }
    }
    
    pub fn get(&mut self, key: &String) -> Option<String> {
        println!("Get: {}", key);
        if self.try_expire(key) { return None }
        
        self.store.get(key).map(|s| s.to_owned())
    }

    /**
     * Returns whether key was set successfully
     */
    pub fn set(&mut self, key: String, value: String, flags: &SetCommandFlags) -> bool {
        println!("Set: {}, {}", key, value);

        if let Some(exist_flag) = &flags.exist_flag {
            let existing_value = &self.get(&key);
            
            if let (SetCommandExistFlag::NX, Some(_)) 
                | (SetCommandExistFlag::XX, None) = (exist_flag, existing_value) {
                return false
            }
        }
        
        if let Some(ttl_flag) = &flags.ttl_flag {
            let maybe_ttl: Option<EpochMillisecond> = match ttl_flag {
                SetCommandTTLFlag::EX(seconds) => {
                    Some(Self::get_unix_time() + (*seconds * 1000))
                },
                SetCommandTTLFlag::PX(milliseconds) => {
                    Some(Self::get_unix_time() + *milliseconds)
                },
                SetCommandTTLFlag::EXAT(seconds) => {
                    Some(*seconds * 1000)
                },
                SetCommandTTLFlag::PXAT(milliseconds) => {
                    Some(*milliseconds)
                },
                SetCommandTTLFlag::KEEPTTL => None,
            };

            if let Some(ttl) = maybe_ttl {
                println!("Setting TTL for {}: {}", key, ttl);
                self.ttl_store.insert(key.to_owned(), ttl);
            } else {
                println!("Keeping existing TTL");
            }
        } else {
            self.ttl_store.remove(&key);
        }

        self.store.insert(key, value);
        true
    }

    /**
     * Returns true if key has expired.
     * Cleans up store passively.
     */
    fn try_expire(&mut self, key: &String) -> bool {
        if let Some(ttl) = self.ttl_store.get(key) {
            if Self::get_unix_time() >= *ttl {
                // Clean up expired key
                println!("Cleaning up for expired key {}: {}", key, ttl);
                self.ttl_store.remove(key);
                self.store.remove(key);

                return true
            }
        }
        false
    }

    fn get_unix_time() -> EpochMillisecond {
        Clock::now()
            .duration_since(UNIX_EPOCH).unwrap()
            .as_millis() as EpochMillisecond
    }
}
