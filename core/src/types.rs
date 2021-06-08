use std::ops::{Deref, DerefMut};

#[derive(Clone, Copy, Debug)]
pub struct Required<T>(Option<T>);

impl<T> Default for Required<T> {
    #[inline]
    fn default() -> Self {
        Self(None)
    }
}

impl<T> Required<T> {
    #[inline]
    #[must_use]
    pub fn is_present(&self) -> bool {
        self.0.is_some()
    }

    #[inline]
    pub fn replace(&mut self, val: T) -> Option<T> {
        self.0.replace(val)
    }

    #[inline]
    #[must_use]
    pub(crate) fn take(&mut self) -> Option<T> {
        self.0.take()
    }
}

impl<T> Deref for Required<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.0.as_ref().unwrap()
    }
}

impl<T> DerefMut for Required<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0.as_mut().unwrap()
    }
}
