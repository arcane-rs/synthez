//! Batteries for working with struct fields.

use std::{
    collections::{BTreeMap, BTreeSet, HashMap, HashSet},
    hash::{BuildHasher, Hash},
    iter, mem,
    ops::{Deref, DerefMut},
};

/// Returns a function checking whether the provided [`Container::is_empty`] and
/// if so, setting the value `parse`d from the provided `Input` into it.
///
/// # Errors
///
/// Propagates the error returned by `parse` function, if any.
///
/// # Example
///
/// Intended to be used as a predicate in a `#[parse]` attribute.
///
/// ```rust
/// # use synthez::{field, parse, ParseAttrs};
/// #
/// #[derive(Default, ParseAttrs)]
/// struct MyAttributes {
///     #[parse(value, fallback = field::if_empty(parse::attr::doc))]
///     description: Option<syn::LitStr>,
/// }
/// ```
pub fn if_empty<V, C, Parser, Input>(
    parse: Parser,
) -> impl FnOnce(&mut C, Input) -> syn::Result<()>
where
    C: Container<V> + ?Sized,
    Parser: FnOnce(Input) -> syn::Result<Option<V>>,
{
    move |container, input| {
        if container.is_empty() {
            if let Some(val) = parse(input)? {
                container.set(val);
            }
        }
        Ok(())
    }
}

/// Container containing field values.
pub trait Container<V> {
    /// Type of values contained in this [`Container`].
    type Value;

    /// Indicates whether this [`Container`] is empty (contains no values).
    #[must_use]
    fn is_empty(&self) -> bool;

    /// Indicates whether the provided `value` is present in this [`Container`].
    #[must_use]
    fn has(&self, value: &V) -> bool;

    /// Replaces the `value` contained in this [`Container`] with the provided
    /// one, and returns the replaced one, if any.
    fn replace(&mut self, value: V) -> Option<V>;

    /// Sets the provided `value` into this  [`Container`], dropping the
    /// previous one, if any.
    fn set(&mut self, value: V) {
        drop(self.replace(value));
    }
}

impl<V> Container<V> for Option<V> {
    type Value = V;

    fn is_empty(&self) -> bool {
        self.is_none()
    }

    fn has(&self, _: &V) -> bool {
        self.is_some()
    }

    fn replace(&mut self, val: V) -> Self {
        Self::replace(self, val)
    }
}

impl<V> Container<V> for Required<V> {
    type Value = V;

    fn is_empty(&self) -> bool {
        !self.is_present()
    }

    fn has(&self, _: &V) -> bool {
        self.is_present()
    }

    fn replace(&mut self, val: V) -> Option<V> {
        Self::replace_with(self, val)
    }
}

impl<V: PartialEq> Container<V> for Vec<V> {
    type Value = V;

    fn is_empty(&self) -> bool {
        Self::is_empty(self)
    }

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

    fn is_empty(&self) -> bool {
        Self::is_empty(self)
    }

    fn has(&self, val: &V) -> bool {
        self.contains(val)
    }

    fn replace(&mut self, val: V) -> Option<V> {
        Self::replace(self, val)
    }
}

impl<V: Ord> Container<V> for BTreeSet<V> {
    type Value = V;

    fn is_empty(&self) -> bool {
        Self::is_empty(self)
    }

    fn has(&self, val: &V) -> bool {
        self.contains(val)
    }

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

    fn is_empty(&self) -> bool {
        Self::is_empty(self)
    }

    fn has(&self, val: &(K, V)) -> bool {
        self.contains_key(&val.0)
    }

    fn replace(&mut self, val: (K, V)) -> Option<(K, V)> {
        let prev = self.remove_entry(&val.0);
        drop(self.insert(val.0, val.1));
        prev
    }
}

impl<K: Ord, V> Container<(K, V)> for BTreeMap<K, V> {
    type Value = (K, V);

    fn is_empty(&self) -> bool {
        Self::is_empty(self)
    }

    fn has(&self, val: &(K, V)) -> bool {
        self.contains_key(&val.0)
    }

    fn replace(&mut self, val: (K, V)) -> Option<(K, V)> {
        let prev = self.remove_entry(&val.0);
        drop(self.insert(val.0, val.1));
        prev
    }
}

/// [`Container`] requiring a field to have a value mandatory.
///
/// It's similar to an [`Option`], but panics on accessing to an absent
/// underlying value. So, is not intended to be created directly in user code,
/// but rather only used for marking types in struct fields, so this library
/// machinery may kick in and take care of correct values creation without
/// introducing any panics.
///
/// Accessing the underlying value, stored in this wrapper, is intended to be
/// done via [`Deref`] and [`DerefMut`] in user code.
#[derive(Clone, Copy, Debug)]
pub struct Required<T>(Option<T>);

#[doc(hidden)]
impl<T> Default for Required<T> {
    fn default() -> Self {
        Self(None)
    }
}

impl<T> Required<T> {
    /// Indicates whether the underlying value is present in this [`Required`]
    /// [`Container`].
    #[must_use]
    pub(crate) const fn is_present(&self) -> bool {
        self.0.is_some()
    }

    /// Replaces the underlying `value` with the given one in this [`Required`]
    /// [`Container`], returning the previous one, if any.
    #[must_use]
    pub(crate) fn replace_with(&mut self, value: T) -> Option<T> {
        self.0.replace(value)
    }

    /// Removes the underlying value from this [`Required`] [`Container`],
    /// returning it, if any.
    #[must_use]
    pub(crate) fn take(&mut self) -> Option<T> {
        self.0.take()
    }

    /// Unwraps this [`Required`] [`Container`] returning the underlying value.
    ///
    /// # Panics
    ///
    /// If this [`Container`] hasn't been initialized properly, so contains no
    /// value.
    #[must_use]
    pub fn into_inner(self) -> T {
        #[allow(clippy::expect_used)]
        self.0.expect("Uninitialized `Required` value")
    }
}

impl<T> Deref for Required<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        #[allow(clippy::expect_used)]
        self.0.as_ref().expect("Uninitialized `Required` value")
    }
}

impl<T> DerefMut for Required<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        #[allow(clippy::expect_used)]
        self.0.as_mut().expect("Uninitialized `Required` value")
    }
}

impl<T> IntoIterator for Required<T> {
    type Item = T;
    type IntoIter = iter::Once<T>;

    fn into_iter(self) -> Self::IntoIter {
        iter::once(self.into_inner())
    }
}

impl<'a, T> IntoIterator for &'a Required<T> {
    type Item = &'a T;
    type IntoIter = iter::Once<&'a T>;

    fn into_iter(self) -> Self::IntoIter {
        iter::once(&**self)
    }
}

impl<'a, T> IntoIterator for &'a mut Required<T> {
    type Item = &'a mut T;
    type IntoIter = iter::Once<&'a mut T>;

    fn into_iter(self) -> Self::IntoIter {
        iter::once(&mut *self)
    }
}
