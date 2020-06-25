use core::sync::atomic::{self, Ordering};
use core::cell::UnsafeCell;
use core::ops::{Deref, DerefMut};

/// Syncronization routines for the Cardinal Operating system.
/// This is mostly based on the Rust standard library std::sync::Mutex, but
/// I've removed the notion of poison for the moment, I don't think it's
/// really going to be possible to implement in a kernel.

pub struct MutexGuard<'a, T: ?Sized + 'a> {
    lock: &'a Mutex<T>,
}

pub struct Mutex<T: ?Sized> {
    lock: atomic::AtomicUsize,
    value: UnsafeCell<T>,
}

impl<T> Mutex<T> {
    pub const fn new(value: T) -> Self {
        Mutex {
            lock: atomic::AtomicUsize::new(0),
            value: UnsafeCell::new(value),
        }
    }
}

impl<T: ?Sized> Mutex<T> {
    /// Acquires a mutex.
    #[must_use]
    pub fn lock(&self) -> MutexGuard<'_, T> {
        match self.try_lock() {
            Ok(a) => a,
            Err(_) => panic!(),
        }
    }

    pub fn try_lock(&self) -> Result<MutexGuard<'_, T>, ()> {
        match self.lock.compare_exchange(
            0, 1, 
            Ordering::Relaxed,
            Ordering::Relaxed
        ) {
            Ok(_) => Ok(unsafe { MutexGuard::new(self) }),
            Err(_) => Err(()),
        }
    }
}

unsafe impl<T: ?Sized + Send> Send for Mutex<T> {}
unsafe impl<T: ?Sized + Send> Sync for Mutex<T> {}

impl<'mutex, T: ?Sized> MutexGuard<'mutex, T> {
    unsafe fn new(lock: &'mutex Mutex<T>) -> MutexGuard<'mutex, T> {
        MutexGuard { lock }
    }
}

impl<T: ?Sized> !Send for MutexGuard<'_, T> {}
unsafe impl <T: ?Sized + Sync> Sync for MutexGuard<'_, T> {}

impl<T: ?Sized> Deref for MutexGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.lock.value.get() }
    }
}

impl<T: ?Sized> DerefMut for MutexGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.lock.value.get() }
    }
}

impl <T: ?Sized> Drop for MutexGuard<'_, T> {
    fn drop(&mut self) {
        self.lock.lock.store(0, Ordering::Relaxed);
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }

    #[test]
    fn basic() {
        let _m = Mutex::new(());
    }

    #[test]
    fn acquire_release() {
        let m = Mutex::new(0);
        {
            let mut guard = m.lock();
            *guard = 10;
            assert!(m.try_lock().is_err());
        }
        let guard2 = m.lock();
        assert_eq!(*guard2, 10);
    }

    #[test]
    #[should_panic]
    fn lock_twice() {
        let m = Mutex::new(0);
        let _g1 = m.lock();
        let _g2 = m.lock();
    }
}
