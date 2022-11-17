use std::time::SystemTime;

/**
 * Clock for SystemTime
 */
pub struct Clock;

#[cfg(not(test))]
impl Clock {
    pub fn now() -> SystemTime {
        SystemTime::now()
    }
}

#[cfg(test)]
pub use test_clock::*;

/**
 * Test implementation for Clock, including mocked system time
 */
#[cfg(test)]
mod test_clock {
    use super::*;

    use std::{ops::Add, sync::{Arc, Mutex, MutexGuard}, time::{Duration, UNIX_EPOCH}};
    use once_cell::sync::Lazy;

    static mut MOCK_TIME: Lazy<Arc<Mutex<Option<SystemTime>>>> = Lazy::new(||
        Arc::new(Mutex::new(None))
    );

    impl Clock {
        pub fn now() -> SystemTime {
            unsafe {
                MOCK_TIME
                    .lock().unwrap()
                    .to_owned().unwrap_or_else(SystemTime::now)
            }
        }

        /**
         * Revert clock back to current system time
         */
        pub fn mock_disable() {
            unsafe {
                *MOCK_TIME.lock().unwrap() = None;
            }
        }

        /**
         * Keeps mocked time frozen to current system time at time of method call
         */
        pub fn mock_freeze() {
            unsafe {
                *MOCK_TIME.lock().unwrap() = Some(SystemTime::now());
            }
        }

        /**
         * Advance mock time
         * If mock time was disabled, advances current system time as new mock time
         */
        pub fn mock_advance(duration: Duration) {
            unsafe {
                let mut time = MOCK_TIME.lock().unwrap();
                *time = Some(
                    (*time)
                        .unwrap_or_else(SystemTime::now)
                        .add(duration)
                );
            }
        }

        /**
         * Sets mock time to a specific epoch
         */
        #[allow(dead_code)]
        pub fn mock_set_time(epoch_millis: u64) {
            unsafe {
                *MOCK_TIME.lock().unwrap() = Some(UNIX_EPOCH.add(Duration::from_millis(epoch_millis)));
            }
        }
    }

    struct MockSessionLock;

    /**
     * Only allows one test to use mocked clock at a time to avoid sync issues
     */
    static mut SESSION_LOCK: Lazy<Arc<Mutex<MockSessionLock>>> = Lazy::new(||
        Arc::new(Mutex::new(MockSessionLock))
    );

    /**
     * Captures lifetime of clock session to avoid tests writing over each's mocked time
     * Use this for any test that relies on SystemTime
     */
    pub struct MockClockSession<'a>(MutexGuard<'a, MockSessionLock>);

    impl <'a> MockClockSession<'a> {
        /**
         * Blocks session if another session is already active 
         */
        pub fn new() -> Self {
            Self(unsafe { SESSION_LOCK.lock().unwrap() })
        }
    }

    impl <'a> Drop for MockClockSession<'a> {
        fn drop(&mut self) {
            Clock::mock_disable()
        }
    }
}
