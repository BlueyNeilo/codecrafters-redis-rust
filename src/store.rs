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
    
    pub fn get(&mut self, key: &str) -> Option<String> {
        println!("Get: {}", key);
        if self.try_expire(key) { return None }
        
        self.store.get(key).map(|s| s.to_owned())
    }

    /**
     * Returns whether key was set successfully
     */
    pub fn set(&mut self, key: &str, value: &str, flags: &SetCommandFlags) -> bool {
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
            self.ttl_store.remove(key);
        }

        self.store.insert(key.to_owned(), value.to_owned());
        true
    }

    /**
     * Returns true if key has expired.
     * Cleans up store passively.
     */
    fn try_expire(&mut self, key: &str) -> bool {
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


#[cfg(test)]
mod concurrent_tests {
    use loom::cell::UnsafeCell;
    use loom::sync::{Arc, Mutex};
    use loom::thread::{self, JoinHandle};

    use crate::resp::command::SetCommandFlags;
    use crate::store::RedisStore;

    #[test]
    #[should_panic]
    fn simulate_non_thread_safe_store_writers() {
        // Two writers accessing store through pointers with no thread safety
        loom::model(|| {
            let shared_store = Arc::new(UnsafeCell::new(RedisStore::default()));

            let threads: Vec<_> = (0..2)
                .map(|_| {
                    let store = shared_store.clone();

                    thread::spawn(move || unsafe {
                        store.get_mut().deref().set("buggy_concurrent_key", "assigned", &SetCommandFlags::default());
                    })
                })
                .collect();
            
            join_all(threads);

            unsafe {
                assert_eq!(Some("assigned".to_owned()), shared_store.get_mut().deref().get("buggy_concurrent_key"));
            }
        });
    }

    #[test]
    fn simulate_thread_safe_store_writers() {
        // Two writers accessing store through pointers with thread safety
        loom::model(|| {
            let shared_store = Arc::new(Mutex::new(RedisStore::default()));

            let threads: Vec<_> = (0..2)
                .map(|_| {
                    let store = shared_store.clone();

                    thread::spawn(move || {
                        store.lock().unwrap().set("concurrent_key", "assigned", &SetCommandFlags::default());
                    })
                })
                .collect();
            
            join_all(threads);

            assert_eq!(Some("assigned".to_owned()), shared_store.lock().unwrap().get("concurrent_key"));
        });
    }


    #[test]
    #[should_panic]
    fn simulate_non_thread_safe_store_reader_writer() {
        // A reader and writer accessing store through pointers with no thread safety
        loom::model(|| {
            let shared_store = Arc::new(UnsafeCell::new(RedisStore::default()));

            let store_writer = shared_store.clone();
            let store_reader = shared_store.clone();

            let threads: Vec<_> = vec![
                // Writer
                thread::spawn(move || unsafe {
                    store_writer.get_mut().deref().set("buggy_concurrent_key", "assigned", &SetCommandFlags::default());
                }),
                // Reader
                thread::spawn(move || unsafe {
                    store_reader.get_mut().deref().get("buggy_concurrent_key");
                }),
            ];
            
            join_all(threads);

            unsafe {
                assert_eq!(Some("assigned".to_owned()), shared_store.get_mut().deref().get("buggy_concurrent_key"));
            }
        });
    }

    #[test]
    fn simulate_thread_safe_store_reader_writer() {
        // A reader and writer accessing store through pointers with thread safety
        loom::model(|| {
            let shared_store = Arc::new(Mutex::new(RedisStore::default()));

            let store_reader = shared_store.clone();
            let store_writer = shared_store.clone();

            let threads: Vec<_> = vec![
                // Writer
                thread::spawn(move || {
                    store_writer.lock().unwrap().set("concurrent_key", "assigned", &SetCommandFlags::default());
                }),
                // Reader
                thread::spawn(move || {
                    store_reader.lock().unwrap().get("concurrent_key");
                }),
            ];
            
            join_all(threads);

            assert_eq!(Some("assigned".to_owned()), shared_store.lock().unwrap().get("concurrent_key"));
        });
    }

    fn join_all(threads: Vec<JoinHandle<()>>) {
        threads.into_iter().for_each(|thread| thread.join().unwrap());
    }
}
