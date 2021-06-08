use std::{
    collections::{BTreeMap, BTreeSet, HashMap, HashSet},
    hash::{BuildHasher, Hash},
    mem,
};

use crate::Required;

pub trait Container<V> {
    type Value;

    #[must_use]
    fn is_empty(&self) -> bool;

    #[must_use]
    fn has(&self, val: &V) -> bool;

    fn replace(&mut self, val: V) -> Option<V>;

    #[inline]
    fn set(&mut self, val: V) {
        drop(self.replace(val))
    }
}

impl<V> Container<V> for Option<V> {
    type Value = V;

    #[must_use]
    fn is_empty(&self) -> bool {
        self.is_none()
    }

    #[inline]
    fn has(&self, _: &V) -> bool {
        self.is_some()
    }

    #[inline]
    fn replace(&mut self, val: V) -> Option<V> {
        Self::replace(self, val)
    }
}

impl<V> Container<V> for Required<V> {
    type Value = V;

    #[must_use]
    fn is_empty(&self) -> bool {
        !self.is_present()
    }

    #[inline]
    fn has(&self, _: &V) -> bool {
        self.is_present()
    }

    #[inline]
    fn replace(&mut self, val: V) -> Option<V> {
        Self::replace(self, val)
    }
}

impl<V: PartialEq> Container<V> for Vec<V> {
    type Value = V;

    #[must_use]
    fn is_empty(&self) -> bool {
        Self::is_empty(self)
    }

    #[inline]
    fn has(&self, val: &V) -> bool {
        self.contains(val)
    }

    fn replace(&mut self, val: V) -> Option<V> {
        if let Some(old) = self.iter_mut().find(|v| *v == &val) {
            Some(mem::replace(old, val))
        } else {
            self.push(val);
            None
        }
    }
}

impl<V, S> Container<V> for HashSet<V, S>
where
    V: Eq + Hash,
    S: BuildHasher,
{
    type Value = V;

    #[must_use]
    fn is_empty(&self) -> bool {
        Self::is_empty(self)
    }

    #[inline]
    fn has(&self, val: &V) -> bool {
        self.contains(val)
    }

    #[inline]
    fn replace(&mut self, val: V) -> Option<V> {
        Self::replace(self, val)
    }
}

impl<V: Ord> Container<V> for BTreeSet<V> {
    type Value = V;

    #[must_use]
    fn is_empty(&self) -> bool {
        Self::is_empty(self)
    }

    #[inline]
    fn has(&self, val: &V) -> bool {
        self.contains(val)
    }

    #[inline]
    fn replace(&mut self, val: V) -> Option<V> {
        Self::replace(self, val)
    }
}

impl<K, V, S> Container<(K, V)> for HashMap<K, V, S>
where
    K: Eq + Hash,
    S: BuildHasher,
{
    type Value = (K, V);

    #[must_use]
    fn is_empty(&self) -> bool {
        Self::is_empty(self)
    }

    #[inline]
    fn has(&self, val: &(K, V)) -> bool {
        self.contains_key(&val.0)
    }

    #[inline]
    fn replace(&mut self, val: (K, V)) -> Option<(K, V)> {
        let prev = self.remove_entry(&val.0);
        drop(self.insert(val.0, val.1));
        prev
    }
}

impl<K: Ord, V> Container<(K, V)> for BTreeMap<K, V> {
    type Value = (K, V);

    #[must_use]
    fn is_empty(&self) -> bool {
        Self::is_empty(self)
    }

    #[inline]
    fn has(&self, val: &(K, V)) -> bool {
        self.contains_key(&val.0)
    }

    #[inline]
    fn replace(&mut self, val: (K, V)) -> Option<(K, V)> {
        let prev = self.remove_entry(&val.0);
        drop(self.insert(val.0, val.1));
        prev
    }
}

pub fn if_empty<V, F, C, I>(
    parse: F,
    container: &mut C,
    input: I,
) -> syn::Result<()>
where
    C: Container<V> + ?Sized,
    F: FnOnce(I) -> syn::Result<Option<V>>,
{
    if container.is_empty() {
        if let Some(val) = parse(input)? {
            drop(container.replace(val));
        }
    }
    Ok(())
}
