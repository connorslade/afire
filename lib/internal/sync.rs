use std::sync::{Condvar, Mutex, MutexGuard, RwLock, RwLockReadGuard, RwLockWriteGuard};

/// Allows locking a mutex even if it's poisoned.
pub trait ForceLockMutex<T> {
    /// Referer to [`Mutex::lock`] documentation.
    /// This function will return the inner value of the mutex if it's poisoned.
    fn force_lock(&self) -> MutexGuard<T>;
}

impl<T> ForceLockMutex<T> for Mutex<T> {
    fn force_lock(&self) -> MutexGuard<T> {
        match self.lock() {
            Ok(i) => i,
            Err(e) => e.into_inner(),
        }
    }
}

/// Allows reading or writing a RwLock even if it's poisoned.
pub trait ForceLockRwLock<T> {
    /// Referer to [`RwLock::read`] documentation.
    /// This function will return the inner value of the RwLock if it's poisoned.
    fn force_read(&self) -> RwLockReadGuard<'_, T>;
    /// Referer to [`RwLock::write`] documentation.
    /// This function will return the inner value of the RwLock if it's poisoned.
    fn force_write(&self) -> RwLockWriteGuard<'_, T>;
}

impl<T> ForceLockRwLock<T> for RwLock<T> {
    fn force_read(&self) -> RwLockReadGuard<'_, T> {
        match self.read() {
            Ok(i) => i,
            Err(e) => e.into_inner(),
        }
    }

    fn force_write(&self) -> RwLockWriteGuard<'_, T> {
        match self.write() {
            Ok(i) => i,
            Err(e) => e.into_inner(),
        }
    }
}

/// A barrier that can only be passed once.
/// It also only blocks the thread that calls `wait` and not all threads.
pub struct SingleBarrier {
    locked: Mutex<bool>,
    condvar: Condvar,
}

impl SingleBarrier {
    /// Create a new `SingleBarrier`.
    pub fn new() -> Self {
        Self {
            locked: Mutex::new(true),
            condvar: Condvar::new(),
        }
    }

    /// Wait for anu thread to call `unlock`, will block until then.
    /// Re-locks the barrier after it's unlocked.
    pub fn wait(&self) {
        let mut locked = self.locked.force_lock();
        while *locked {
            locked = self.condvar.wait(locked).unwrap();
        }
        *locked = true;
    }

    /// Unlock the barrier.
    pub fn unlock(&self) {
        let mut locked = self.locked.force_lock();
        *locked = false;
        self.condvar.notify_all();
    }

    pub fn reset(&self) {
        let mut locked = self.locked.force_lock();
        *locked = true;
    }
}

impl Default for SingleBarrier {
    fn default() -> Self {
        Self::new()
    }
}
