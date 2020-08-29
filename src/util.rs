use core::ops;

pub trait Tap: Sized {
    fn tap<F: FnOnce(Self) -> T, T>(self, apply: F) -> T {
        apply(self)
    }

    fn tap_mut<F: FnOnce(&mut Self)>(mut self, apply: F) -> Self {
        apply(&mut self);
        self
    }
}

impl<T> Tap for T {}

#[derive(Debug)]
pub struct Mutex<T>(spin::Mutex<T>);

impl<T> Mutex<T> {
    pub const fn new(inner: T) -> Self {
        Mutex(spin::Mutex::new(inner))
    }
}

impl<T> ops::Deref for Mutex<T> {
    type Target = spin::Mutex<T>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
