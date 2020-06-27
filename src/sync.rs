use core::cell::UnsafeCell;
use core::ops::{Deref, DerefMut};
use core::sync::atomic::{self, Ordering};

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
        match self
            .lock
            .compare_exchange(0, 1, Ordering::Relaxed, Ordering::Relaxed)
        {
            Ok(_) => Ok(unsafe { MutexGuard::new(self) }),
            Err(_) => Err(()),
        }
    }

    pub fn into_inner(self) -> T
    where
        T: Sized,
    {
        self.value.into_inner()
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
unsafe impl<T: ?Sized + Sync> Sync for MutexGuard<'_, T> {}

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

impl<T: ?Sized> Drop for MutexGuard<'_, T> {
    fn drop(&mut self) {
        self.lock.lock.store(0, Ordering::Relaxed);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use alloc::sync::Arc;
    use core::sync::atomic::{AtomicUsize, Ordering};

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

    // Several tests adopted from Rust standard library Mutex tests
    // https://doc.rust-lang.org/src/std/sync/mutex.rs.html

    #[test]
    fn smoke() {
        let m = Mutex::new(());
        drop(m.lock());
        drop(m.lock());
    }

    #[derive(Eq, PartialEq, Debug)]
    struct NonCopy(i32);

    #[test]
    fn try_lock() {
        let m = Mutex::new(());
        *m.try_lock().unwrap() = ();
    }

    #[test]
    fn test_into_inner() {
        let m = Mutex::new(NonCopy(10));
        assert_eq!(m.into_inner(), NonCopy(10));
    }

    #[test]
    fn test_into_inner_drop() {
        struct Foo(Arc<AtomicUsize>);
        impl Drop for Foo {
            fn drop(&mut self) {
                self.0.fetch_add(1, Ordering::SeqCst);
            }
        }
        let num_drops = Arc::new(AtomicUsize::new(0));
        let m = Mutex::new(Foo(num_drops.clone()));
        assert_eq!(num_drops.load(Ordering::SeqCst), 0);
        {
            let _inner = m.into_inner();
            assert_eq!(num_drops.load(Ordering::SeqCst), 0);
        }
        assert_eq!(num_drops.load(Ordering::SeqCst), 1);
    }
}
