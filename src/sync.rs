// use core::sync::atomic::A

use core::cell::UnsafeCell;
use core::ops::Deref;
use core::ops::DerefMut;
use core::sync::atomic::AtomicBool;
use core::sync::atomic::Ordering;

pub struct SpinLock<T> {
    lock: AtomicBool,
    inner: UnsafeCell<T>,
}

unsafe impl<T: Send> Sync for SpinLock<T> {}
unsafe impl<T: Send> Send for SpinLock<T> {}

impl<T> SpinLock<T> {
    pub const fn new(inner: T) -> Self {
        Self {
            lock: AtomicBool::new(false),
            inner: UnsafeCell::new(inner),
        }
    }

    pub fn lock(&self) -> SpinLockGuard<T> {
        loop {
            while self.lock.load(Ordering::Relaxed) {
                crate::pause();
            }

            if let Ok(false) =
                self.lock
                    .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
            {
                break;
            }
        }

        SpinLockGuard {
            lock: self,
            inner: unsafe { self.inner.get().as_mut().unwrap() },
        }
    }
}

pub struct SpinLockGuard<'a, T> {
    lock: &'a SpinLock<T>,
    inner: &'a mut T,
}

impl<T> Deref for SpinLockGuard<'_, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        self.inner
    }
}

impl<T> DerefMut for SpinLockGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.inner
    }
}

impl<T> Drop for SpinLockGuard<'_, T> {
    fn drop(&mut self) {
        self.lock.lock.store(false, Ordering::Release);
    }
}
