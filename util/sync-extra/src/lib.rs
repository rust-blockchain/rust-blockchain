use std::sync::{Mutex, MutexGuard, RwLock, RwLockReadGuard, RwLockWriteGuard};

pub trait MutexExtra {
    type Value: ?Sized;

    fn lock_unwrap(&self) -> MutexGuard<'_, Self::Value>;
}

impl<T: ?Sized> MutexExtra for Mutex<T> {
    type Value = T;

    fn lock_unwrap(&self) -> MutexGuard<'_, Self::Value> {
        self.lock().expect("lock is poisioned")
    }
}

pub trait RwLockExtra {
    type Value: ?Sized;

    fn read_unwrap(&self) -> RwLockReadGuard<'_, Self::Value>;
    fn write_unwrap(&self) -> RwLockWriteGuard<'_, Self::Value>;
}

impl<T: ?Sized> RwLockExtra for RwLock<T> {
    type Value = T;

    fn read_unwrap(&self) -> RwLockReadGuard<'_, Self::Value> {
        self.read().expect("lock is poisioned")
    }

    fn write_unwrap(&self) -> RwLockWriteGuard<'_, Self::Value> {
        self.write().expect("lock is poisioned")
    }
}
