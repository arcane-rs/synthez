//! Machinery for parsing [`syn::Attribute`]s into a custom defined struct.

use proc_macro2::Span;
use syn::{parse::Parse, spanned::Spanned};

use crate::has;

use super::err;

#[doc(inline)]
pub use self::{dedup::Dedup, kind::Kind, validate::Validate};

/// [`Parse`]ing of [`syn::Attribute`]s into a custom defined struct.
pub trait Attrs: Default + Parse {
    /// Tries to merge two sets of parsed attributes into a single one,
    /// reporting about duplicates, if any.
    ///
    /// # Errors
    ///
    /// If merging cannot be performed.
    fn try_merge(self, another: Self) -> syn::Result<Self>;

    /// Validates these parsed attributes to meet additional invariants, if
    /// required.
    ///
    /// The provided string contains name of the parsed [`syn::Attribute`], and
    /// the provided [`Span`] refers to the item this [`syn::Attribute`] is
    /// applied to. Use them to make reported errors well descriptive.
    ///
    /// # Errors
    ///
    /// If validation fails.
    #[inline]
    fn validate(&self, _: &str, _: Span) -> syn::Result<()> {
        Ok(())
    }

    /// Falls back to another values from [`syn::Attribute`]s, if required.
    ///
    /// # Errors
    ///
    /// If retrieving fallback values fails.
    #[inline]
    fn fallback(&mut self, _: &[syn::Attribute]) -> syn::Result<()> {
        Ok(())
    }

    /// Parses this structure from the [`syn::Attribute`]s with the given `name`
    /// and contained in the given `item`.
    ///
    /// If multiple [`syn::Attribute`]s occur with the same `name` then they all
    /// are parsed separately and then [`Attrs::try_merge`]d.
    ///
    /// If none [`syn::Attribute`]s occur with the given `name` then [`Default`]
    /// value is returned, modulo [`Attrs::validate`].
    ///
    /// # Errors
    ///
    /// - If [`Parse`]ing of this [`Attrs`] fails.
    /// - If either [`Attrs::try_merge()`], [`Attrs::validate()`] or
    ///   [`Attrs::fallback()`] fails.
    fn parse_attrs<T>(name: &str, item: &T) -> syn::Result<Self>
    where
        T: has::Attrs + Spanned,
    {
        let attrs = item.attrs();
        filter_by_name(name, attrs)
            .map(syn::Attribute::parse_args)
            .try_fold(Self::default(), |prev, curr| prev.try_merge(curr?))
            .and_then(|mut parsed| {
                parsed.fallback(attrs)?;
                parsed.validate(name, item.span())?;
                Ok(parsed)
            })
    }
}

/// Filters the given `attrs` to contain [`syn::Attribute`]s only with the given
/// `name`.
#[inline]
pub fn filter_by_name<'n: 'ret, 'a: 'ret, 'ret>(
    name: &'n str,
    attrs: &'a [syn::Attribute],
) -> impl Iterator<Item = &'a syn::Attribute> + 'ret {
    attrs.iter().filter(move |attr| path_eq_single(&attr.path, name))
}

/// Compares the given `path` with the one-segment string `value` to be equal.
#[inline]
#[must_use]
fn path_eq_single(path: &syn::Path, value: &str) -> bool {
    path.segments.len() == 1 && path.segments[0].ident == value
}

pub mod field {
    //! Batteries for working with [`Attrs`]' fields.
    //!
    //! [`Attrs`]: super::Attrs

    use sealed::sealed;

    use crate::field;

    use super::{Dedup, Kind};

    /// Applying a value to a [`field::Container`] according to a parsing
    /// [`Kind`] and [`Dedup`]lication strategy.
    pub trait TryApply<V, K: Kind + ?Sized, D: Dedup + ?Sized>:
        field::Container<V>
    {
        /// Applies the provided `value` to this [`field::Container`].
        ///
        /// # Errors
        ///
        /// If this [`field::Container`] refuses to apply the `value` according
        /// to the [`Dedup`]lication strategy.
        fn try_apply(&mut self, value: V) -> syn::Result<()>;
    }

    /// Applying a value to a [`field::Container`] according to a parsing
    /// [`Kind`] and [`Dedup`]lication strategy from another
    /// [`field::Container`].
    pub trait TryApplySelf<V, K: Kind + ?Sized, D: Dedup + ?Sized>:
        TryApply<V, K, D>
    {
        /// Applies the value extracted from `another` [`field::Container`] to
        /// this [`field::Container`].
        ///
        /// # Errors
        ///
        /// If this [`field::Container`] refuses to apply the extracted value
        /// according to the [`Dedup`]lication strategy.
        fn try_apply_self(&mut self, another: Self) -> syn::Result<()>;
    }

    mod option {
        //! [`TryApply`] impls for [`Option`].

        use syn::spanned::Spanned;

        use crate::field::Container as _;

        use super::{
            super::{dedup, err, kind, Dedup, Kind},
            TryApply, TryApplySelf,
        };

        impl<V, K> TryApply<V, K, dedup::Unique> for Option<V>
        where
            V: Spanned,
            K: Kind + kind::Single + ?Sized,
        {
            fn try_apply(&mut self, val: V) -> syn::Result<()> {
                if self.has(&val) {
                    return Err(err::dup_attr_arg(&val));
                }
                self.set(val);
                Ok(())
            }
        }

        impl<V, K> TryApply<V, K, dedup::First> for Option<V>
        where
            K: Kind + kind::Single + ?Sized,
        {
            fn try_apply(&mut self, val: V) -> syn::Result<()> {
                if !self.has(&val) {
                    self.set(val);
                }
                Ok(())
            }
        }

        impl<V, K> TryApply<V, K, dedup::Last> for Option<V>
        where
            K: Kind + kind::Single + ?Sized,
        {
            fn try_apply(&mut self, val: V) -> syn::Result<()> {
                self.set(val);
                Ok(())
            }
        }

        impl<V, K, D> TryApplySelf<V, K, D> for Option<V>
        where
            V: Spanned,
            K: Kind + kind::Single + ?Sized,
            D: Dedup + ?Sized,
            Self: TryApply<V, K, D>,
        {
            fn try_apply_self(&mut self, another: Self) -> syn::Result<()> {
                if let Some(val) = another {
                    self.try_apply(val)?;
                }
                Ok(())
            }
        }
    }

    mod required {
        //! [`TryApply`] impls for [`Required`].

        use syn::spanned::Spanned;

        use crate::{field::Container as _, Required};

        use super::{
            super::{dedup, err, kind, Dedup, Kind},
            TryApply, TryApplySelf,
        };

        impl<V, K> TryApply<V, K, dedup::Unique> for Required<V>
        where
            V: Spanned,
            K: Kind + kind::Single + ?Sized,
        {
            fn try_apply(&mut self, val: V) -> syn::Result<()> {
                if self.has(&val) {
                    return Err(err::dup_attr_arg(&val));
                }
                self.set(val);
                Ok(())
            }
        }

        impl<V, K> TryApply<V, K, dedup::First> for Required<V>
        where
            K: Kind + kind::Single + ?Sized,
        {
            fn try_apply(&mut self, val: V) -> syn::Result<()> {
                if !self.has(&val) {
                    self.set(val);
                }
                Ok(())
            }
        }

        impl<V, K> TryApply<V, K, dedup::Last> for Required<V>
        where
            K: Kind + kind::Single + ?Sized,
        {
            fn try_apply(&mut self, val: V) -> syn::Result<()> {
                self.set(val);
                Ok(())
            }
        }

        impl<V, K, D> TryApplySelf<V, K, D> for Required<V>
        where
            V: Spanned,
            K: Kind + kind::Single + ?Sized,
            D: Dedup + ?Sized,
            Self: TryApply<V, K, D>,
        {
            fn try_apply_self(&mut self, mut another: Self) -> syn::Result<()> {
                if let Some(val) = another.take() {
                    self.try_apply(val)?;
                }
                Ok(())
            }
        }
    }

    mod vec {
        //! [`TryApply`] impls for [`Vec`].

        use syn::spanned::Spanned;

        use crate::field::Container as _;

        use super::{
            super::{dedup, err, kind, Dedup},
            TryApply, TryApplySelf,
        };

        impl<V> TryApply<V, kind::Nested, dedup::Unique> for Vec<V>
        where
            V: Spanned + PartialEq,
        {
            fn try_apply(&mut self, val: V) -> syn::Result<()> {
                if self.has(&val) {
                    return Err(err::dup_attr_arg(&val));
                }
                self.set(val);
                Ok(())
            }
        }

        impl<V: PartialEq> TryApply<V, kind::Nested, dedup::First> for Vec<V> {
            fn try_apply(&mut self, val: V) -> syn::Result<()> {
                if !self.has(&val) {
                    self.set(val);
                }
                Ok(())
            }
        }

        impl<V: PartialEq> TryApply<V, kind::Nested, dedup::Last> for Vec<V> {
            fn try_apply(&mut self, val: V) -> syn::Result<()> {
                self.set(val);
                Ok(())
            }
        }

        impl<V, D> TryApplySelf<V, kind::Nested, D> for Vec<V>
        where
            V: Spanned + PartialEq,
            D: Dedup + ?Sized,
            Self: TryApply<V, kind::Nested, D>,
        {
            fn try_apply_self(&mut self, another: Self) -> syn::Result<()> {
                for val in another {
                    self.try_apply(val)?;
                }
                Ok(())
            }
        }

        impl<V> TryApply<V, kind::Value, dedup::Unique> for Vec<V>
        where
            V: Spanned + PartialEq,
        {
            fn try_apply(&mut self, val: V) -> syn::Result<()> {
                if self.has(&val) {
                    return Err(err::dup_attr_arg(&val));
                }
                self.set(val);
                Ok(())
            }
        }

        impl<V: PartialEq> TryApply<V, kind::Value, dedup::First> for Vec<V> {
            fn try_apply(&mut self, val: V) -> syn::Result<()> {
                if !self.has(&val) {
                    self.set(val);
                }
                Ok(())
            }
        }

        impl<V: PartialEq> TryApply<V, kind::Value, dedup::Last> for Vec<V> {
            fn try_apply(&mut self, val: V) -> syn::Result<()> {
                self.set(val);
                Ok(())
            }
        }

        impl<V, D> TryApplySelf<V, kind::Value, D> for Vec<V>
        where
            V: Spanned + PartialEq,
            D: Dedup + ?Sized,
            Self: TryApply<V, kind::Value, D>,
        {
            fn try_apply_self(&mut self, another: Self) -> syn::Result<()> {
                for val in another {
                    self.try_apply(val)?;
                }
                Ok(())
            }
        }
    }

    mod hashset {
        //! [`TryApply`] impls for [`HashSet`].

        use std::{
            collections::HashSet,
            hash::{BuildHasher, Hash},
        };

        use syn::spanned::Spanned;

        use crate::field::Container as _;

        use super::{
            super::{dedup, err, kind, Dedup},
            TryApply, TryApplySelf,
        };

        impl<V, S> TryApply<V, kind::Nested, dedup::Unique> for HashSet<V, S>
        where
            V: Spanned + Eq + Hash,
            S: BuildHasher,
        {
            fn try_apply(&mut self, val: V) -> syn::Result<()> {
                if self.has(&val) {
                    return Err(err::dup_attr_arg(&val));
                }
                self.set(val);
                Ok(())
            }
        }

        impl<V, S> TryApply<V, kind::Nested, dedup::First> for HashSet<V, S>
        where
            V: Eq + Hash,
            S: BuildHasher,
        {
            fn try_apply(&mut self, val: V) -> syn::Result<()> {
                if !self.has(&val) {
                    self.set(val);
                }
                Ok(())
            }
        }

        impl<V, S> TryApply<V, kind::Nested, dedup::Last> for HashSet<V, S>
        where
            V: Eq + Hash,
            S: BuildHasher,
        {
            fn try_apply(&mut self, val: V) -> syn::Result<()> {
                self.set(val);
                Ok(())
            }
        }

        impl<V, S, D> TryApplySelf<V, kind::Nested, D> for HashSet<V, S>
        where
            V: Spanned + Eq + Hash,
            D: Dedup + ?Sized,
            S: BuildHasher,
            Self: TryApply<V, kind::Nested, D>,
        {
            fn try_apply_self(&mut self, another: Self) -> syn::Result<()> {
                for val in another {
                    self.try_apply(val)?;
                }
                Ok(())
            }
        }

        impl<V, S> TryApply<V, kind::Value, dedup::Unique> for HashSet<V, S>
        where
            V: Spanned + Eq + Hash,
            S: BuildHasher,
        {
            fn try_apply(&mut self, val: V) -> syn::Result<()> {
                if self.has(&val) {
                    return Err(err::dup_attr_arg(&val));
                }
                self.set(val);
                Ok(())
            }
        }

        impl<V, S> TryApply<V, kind::Value, dedup::First> for HashSet<V, S>
        where
            V: Eq + Hash,
            S: BuildHasher,
        {
            fn try_apply(&mut self, val: V) -> syn::Result<()> {
                if !self.has(&val) {
                    self.set(val);
                }
                Ok(())
            }
        }

        impl<V, S> TryApply<V, kind::Value, dedup::Last> for HashSet<V, S>
        where
            V: Eq + Hash,
            S: BuildHasher,
        {
            fn try_apply(&mut self, val: V) -> syn::Result<()> {
                self.set(val);
                Ok(())
            }
        }

        impl<V, S, D> TryApplySelf<V, kind::Value, D> for HashSet<V, S>
        where
            V: Spanned + Eq + Hash,
            D: Dedup + ?Sized,
            S: BuildHasher,
            Self: TryApply<V, kind::Value, D>,
        {
            fn try_apply_self(&mut self, another: Self) -> syn::Result<()> {
                for val in another {
                    self.try_apply(val)?;
                }
                Ok(())
            }
        }
    }

    mod btreeset {
        //! [`TryApply`] impls for [`BTreeSet`].

        use std::collections::BTreeSet;

        use syn::spanned::Spanned;

        use crate::field::Container as _;

        use super::{
            super::{dedup, err, kind, Dedup},
            TryApply, TryApplySelf,
        };

        impl<V> TryApply<V, kind::Nested, dedup::Unique> for BTreeSet<V>
        where
            V: Spanned + Ord,
        {
            fn try_apply(&mut self, val: V) -> syn::Result<()> {
                if self.has(&val) {
                    return Err(err::dup_attr_arg(&val));
                }
                self.set(val);
                Ok(())
            }
        }

        impl<V: Ord> TryApply<V, kind::Nested, dedup::First> for BTreeSet<V> {
            fn try_apply(&mut self, val: V) -> syn::Result<()> {
                if !self.has(&val) {
                    self.set(val);
                }
                Ok(())
            }
        }

        impl<V: Ord> TryApply<V, kind::Nested, dedup::Last> for BTreeSet<V> {
            fn try_apply(&mut self, val: V) -> syn::Result<()> {
                self.set(val);
                Ok(())
            }
        }

        impl<V, D> TryApplySelf<V, kind::Nested, D> for BTreeSet<V>
        where
            V: Spanned + Ord,
            D: Dedup + ?Sized,
            Self: TryApply<V, kind::Nested, D>,
        {
            fn try_apply_self(&mut self, another: Self) -> syn::Result<()> {
                for val in another {
                    self.try_apply(val)?;
                }
                Ok(())
            }
        }

        impl<V> TryApply<V, kind::Value, dedup::Unique> for BTreeSet<V>
        where
            V: Spanned + Ord,
        {
            fn try_apply(&mut self, val: V) -> syn::Result<()> {
                if self.has(&val) {
                    return Err(err::dup_attr_arg(&val));
                }
                self.set(val);
                Ok(())
            }
        }

        impl<V: Ord> TryApply<V, kind::Value, dedup::First> for BTreeSet<V> {
            fn try_apply(&mut self, val: V) -> syn::Result<()> {
                if !self.has(&val) {
                    self.set(val);
                }
                Ok(())
            }
        }

        impl<V: Ord> TryApply<V, kind::Value, dedup::Last> for BTreeSet<V> {
            fn try_apply(&mut self, val: V) -> syn::Result<()> {
                self.set(val);
                Ok(())
            }
        }

        impl<V, D> TryApplySelf<V, kind::Value, D> for BTreeSet<V>
        where
            V: Spanned + Ord,
            D: Dedup + ?Sized,
            Self: TryApply<V, kind::Value, D>,
        {
            fn try_apply_self(&mut self, another: Self) -> syn::Result<()> {
                for val in another {
                    self.try_apply(val)?;
                }
                Ok(())
            }
        }
    }

    mod hashmap {
        //! [`TryApply`] impls for [`HashMap`].

        use std::{
            collections::HashMap,
            hash::{BuildHasher, Hash},
        };

        use syn::spanned::Spanned;

        use crate::field::Container as _;

        use super::{
            super::{dedup, err, kind, Dedup},
            TryApply, TryApplySelf,
        };

        impl<K: Spanned + Eq + Hash, V, S: BuildHasher>
            TryApply<(K, V), kind::Map, dedup::Unique> for HashMap<K, V, S>
        {
            fn try_apply(&mut self, val: (K, V)) -> syn::Result<()> {
                if self.has(&val) {
                    return Err(err::dup_attr_arg(&val.0));
                }
                self.set(val);
                Ok(())
            }
        }

        impl<K: Eq + Hash, V, S: BuildHasher>
            TryApply<(K, V), kind::Map, dedup::First> for HashMap<K, V, S>
        {
            fn try_apply(&mut self, val: (K, V)) -> syn::Result<()> {
                if !self.has(&val) {
                    self.set(val);
                }
                Ok(())
            }
        }

        impl<K: Eq + Hash, V, S: BuildHasher>
            TryApply<(K, V), kind::Map, dedup::Last> for HashMap<K, V, S>
        {
            fn try_apply(&mut self, val: (K, V)) -> syn::Result<()> {
                self.set(val);
                Ok(())
            }
        }

        impl<K, V, D, S> TryApplySelf<(K, V), kind::Map, D> for HashMap<K, V, S>
        where
            K: Spanned + Eq + Hash,
            D: Dedup + ?Sized,
            S: BuildHasher,
            Self: TryApply<(K, V), kind::Map, D>,
        {
            fn try_apply_self(&mut self, another: Self) -> syn::Result<()> {
                for val in another {
                    self.try_apply(val)?;
                }
                Ok(())
            }
        }
    }

    mod btreemap {
        //! [`TryApply`] impls for [`BTreeMap`].

        use std::collections::BTreeMap;

        use syn::spanned::Spanned;

        use crate::field::Container as _;

        use super::{
            super::{dedup, err, kind, Dedup},
            TryApply, TryApplySelf,
        };

        impl<K, V> TryApply<(K, V), kind::Map, dedup::Unique> for BTreeMap<K, V>
        where
            K: Spanned + Ord,
        {
            fn try_apply(&mut self, val: (K, V)) -> syn::Result<()> {
                if self.has(&val) {
                    return Err(err::dup_attr_arg(&val.0));
                }
                self.set(val);
                Ok(())
            }
        }

        impl<K, V> TryApply<(K, V), kind::Map, dedup::First> for BTreeMap<K, V>
        where
            K: Ord,
        {
            fn try_apply(&mut self, val: (K, V)) -> syn::Result<()> {
                if !self.has(&val) {
                    self.set(val);
                }
                Ok(())
            }
        }

        impl<K, V> TryApply<(K, V), kind::Map, dedup::Last> for BTreeMap<K, V>
        where
            K: Ord,
        {
            fn try_apply(&mut self, val: (K, V)) -> syn::Result<()> {
                self.set(val);
                Ok(())
            }
        }

        impl<K, V, D> TryApplySelf<(K, V), kind::Map, D> for BTreeMap<K, V>
        where
            K: Spanned + Ord,
            D: Dedup + ?Sized,
            Self: TryApply<(K, V), kind::Map, D>,
        {
            fn try_apply_self(&mut self, another: Self) -> syn::Result<()> {
                for val in another {
                    self.try_apply(val)?;
                }
                Ok(())
            }
        }
    }

    /// [`TryApply`] and [`TryApplySelf`] traits' shim allowing to specify a
    /// parsing [`Kind`] and [`Dedup`]lication strategy as method's type
    /// parameters.
    #[sealed]
    pub trait TryMerge<V> {
        /// Merges the provided `value` to this [`field::Container`] with the
        /// specified parsing [`Kind`] and [`Dedup`]lication strategy.
        ///
        /// # Errors
        ///
        /// If this [`field::Container`] refuses to apply the `value` according
        /// to the [`Dedup`]lication strategy.
        fn try_merge<K, D>(&mut self, value: V) -> syn::Result<()>
        where
            Self: TryApply<V, K, D>,
            K: Kind + ?Sized,
            D: Dedup + ?Sized;

        /// Merges the value extracted from `another` [`field::Container`] to
        /// this [`field::Container`] with the specified parsing [`Kind`] and
        /// [`Dedup`]lication strategy.
        ///
        /// # Errors
        ///
        /// If this [`field::Container`] refuses to apply the extracted value
        /// according to the [`Dedup`]lication strategy.
        fn try_merge_self<K, D>(&mut self, another: Self) -> syn::Result<()>
        where
            Self: TryApplySelf<V, K, D> + Sized,
            K: Kind + ?Sized,
            D: Dedup + ?Sized;
    }

    #[sealed]
    impl<T: ?Sized, V> TryMerge<V> for T {
        #[inline]
        fn try_merge<K, D>(&mut self, val: V) -> syn::Result<()>
        where
            Self: TryApply<V, K, D>,
            K: Kind + ?Sized,
            D: Dedup + ?Sized,
        {
            <Self as TryApply<V, K, D>>::try_apply(self, val)
        }

        #[inline]
        fn try_merge_self<K, D>(&mut self, another: Self) -> syn::Result<()>
        where
            Self: TryApplySelf<V, K, D> + Sized,
            K: Kind + ?Sized,
            D: Dedup + ?Sized,
        {
            <Self as TryApplySelf<V, K, D>>::try_apply_self(self, another)
        }
    }
}

pub mod kind {
    //! Kinds of an [`Attrs`]' field parsing.
    //!
    //! [`Attrs`]: super::Attrs

    use sealed::sealed;

    /// Abstracted kind of an [`Attrs`]' field parsing into a
    /// [`field::Container`].
    ///
    /// [`Attrs`]: super::Attrs
    /// [`field::Container`]: crate::field::Container
    #[sealed]
    pub trait Kind {}

    /// [`Kind`]s allowing to parse only a single value of an [`Attrs`]' field.
    #[sealed]
    pub trait Single {}

    /// [`Kind`] defining parsing an [`Attrs`]' field as a simple
    /// [`syn::Ident`].
    ///
    /// ```text
    /// #[attr(ident)]
    /// ```
    ///
    /// [`Attrs`]: super::Attrs
    #[derive(Clone, Copy, Debug)]
    pub enum Ident {}

    #[sealed]
    impl Kind for Ident {}

    #[sealed]
    impl Single for Ident {}

    /// [`Kind`] defining parsing an [`Attrs`]' field as nested [`Attrs`].
    ///
    /// ```text
    /// #[attr(nested(ident, key = val))]
    /// ```
    ///
    /// [`Attrs`]: super::Attrs
    #[derive(Clone, Copy, Debug)]
    pub enum Nested {}

    #[sealed]
    impl Kind for Nested {}

    #[sealed]
    impl Single for Nested {}

    /// [`Kind`] defining parsing an [`Attrs`]' field as values behind a
    /// [`syn::Ident`].
    ///
    /// ```text
    /// #[attr(ident = value)]
    /// #[attr(ident value)]
    /// #[attr(ident(value1, value2))]
    /// ```
    ///
    /// [`Attrs`]: super::Attrs
    #[derive(Clone, Copy, Debug)]
    pub enum Value {}

    #[sealed]
    impl Kind for Value {}

    #[sealed]
    impl Single for Value {}

    /// [`Kind`] defining parsing an [`Attrs`]' field as key-value pairs behind
    /// a [`syn::Ident`].
    ///
    /// ```text
    /// #[attr(ident key = value)]
    /// ```
    ///
    /// [`Attrs`]: super::Attrs
    #[derive(Clone, Copy, Debug)]
    pub enum Map {}

    #[sealed]
    impl Kind for Map {}
}

pub mod dedup {
    //! Deduplication strategies of an [`Attrs`]' field parsing.
    //!
    //! [`Attrs`]: super::Attrs

    use sealed::sealed;

    /// Abstracted deduplication strategy of an [`Attrs`]' field parsing into a
    /// [`field::Container`].
    ///
    /// [`Attrs`]: super::Attrs
    /// [`field::Container`]: crate::field::Container
    #[sealed]
    pub trait Dedup {}

    /// [`Dedup`]lication strategy allowing only a single value of an [`Attrs`]'
    /// field to appear.
    ///
    /// [`Attrs`]: super::Attrs
    #[derive(Clone, Copy, Debug)]
    pub enum Unique {}

    #[sealed]
    impl Dedup for Unique {}

    /// [`Dedup`]lication strategy picking only the first parsed value of an
    /// [`Attrs`]' field.
    ///
    /// [`Attrs`]: super::Attrs
    #[derive(Clone, Copy, Debug)]
    pub enum First {}

    #[sealed]
    impl Dedup for First {}

    /// [`Dedup`]lication strategy picking only the last parsed value of an
    /// [`Attrs`]' field.
    ///
    /// [`Attrs`]: super::Attrs
    #[derive(Clone, Copy, Debug)]
    pub enum Last {}

    #[sealed]
    impl Dedup for Last {}
}

pub mod validate {
    //! Validation machinery of an [`Attrs`]' field parsing.
    //!
    //! [`Attrs`]: super::Attrs

    use std::{
        collections::{HashMap, HashSet},
        hash::{BuildHasher, Hash},
    };

    use sealed::sealed;

    use crate::Required;

    #[doc(inline)]
    pub use self::rule::Rule;

    /// Validation of a [`Rule`] during an [`Attrs`]' field parsing into a
    /// [`field::Container`].
    ///
    /// [`field::Container`]: crate::field::Container
    pub trait Validate<R: Rule + ?Sized> {
        /// Checks whether the validation [`Rule`] is satisfied.
        #[must_use]
        fn validate(&self) -> bool;
    }

    impl<V> Validate<rule::Provided> for Required<V> {
        #[inline]
        fn validate(&self) -> bool {
            self.is_present()
        }
    }

    impl<V> Validate<rule::Provided> for Option<V> {
        #[inline]
        fn validate(&self) -> bool {
            true
        }
    }

    impl<V, S> Validate<rule::Provided> for HashSet<V, S>
    where
        V: Eq + Hash,
        S: BuildHasher,
    {
        #[inline]
        fn validate(&self) -> bool {
            true
        }
    }

    impl<V> Validate<rule::Provided> for Vec<V> {
        #[inline]
        fn validate(&self) -> bool {
            true
        }
    }

    impl<K, V, S> Validate<rule::Provided> for HashMap<K, V, S>
    where
        K: Eq + Hash,
        S: BuildHasher,
    {
        #[inline]
        fn validate(&self) -> bool {
            true
        }
    }

    /// [`Validate`] trait's shim allowing to specify its [`Rule`] as a method's
    /// type parameter.
    #[sealed]
    pub trait IsValid {
        /// Checks whether the specified validation [`Rule`] is satisfied.
        #[must_use]
        fn is_valid<R: Rule + ?Sized>(&self) -> bool
        where
            Self: Validate<R>;
    }

    #[sealed]
    impl<T: ?Sized> IsValid for T {
        #[inline]
        fn is_valid<R: Rule + ?Sized>(&self) -> bool
        where
            Self: Validate<R>,
        {
            self.validate()
        }
    }

    pub mod rule {
        //! Validation [`Rule`]s of an [`Attrs`]' field parsing.
        //!
        //! [`Attrs`]: super::super::Attrs

        use sealed::sealed;

        /// Abstracted validation rule of an [`Attrs`]' field parsing.
        ///
        /// [`Attrs`]: super::super::Attrs
        #[sealed]
        pub trait Rule {}

        /// Validation [`Rule`] verifying whether an [`Attrs`]' field has been
        /// provided for parsing.
        ///
        /// [`Attrs`]: super::super::Attrs
        #[derive(Clone, Copy, Debug)]
        pub enum Provided {}

        #[sealed]
        impl Rule for Provided {}
    }
}
