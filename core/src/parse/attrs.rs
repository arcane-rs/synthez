use proc_macro2::Span;
use syn::{parse::Parse, spanned::Spanned};

use crate::has;

use super::err;

pub use self::{dedup::Dedup, kind::Kind, validate::Validate};

pub trait Attrs: Default + Parse {
    /// Tries to merge two sets of parsed attributes into a single one,
    /// reporting about duplicates, if any.
    fn try_merge(self, another: Self) -> syn::Result<Self>;

    /// Validates these parsed attributes to meet additional invariants, if
    /// required.
    #[inline]
    fn validate(&self, _: &str, _: Span) -> syn::Result<()> {
        Ok(())
    }

    /// Falls back to another values from attributes, if required.
    #[inline]
    fn fallback(&mut self, _: &[syn::Attribute]) -> syn::Result<()> {
        Ok(())
    }

    fn parse_attrs<T>(name: &str, item: &T) -> syn::Result<Self>
    where
        T: has::Attrs + Spanned,
    {
        let attrs = item.attrs();
        filter(name, attrs)
            .map(|attr| attr.parse_args())
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
fn filter<'n: 'ret, 'a: 'ret, 'ret>(
    name: &'n str,
    attrs: &'a [syn::Attribute],
) -> impl Iterator<Item = &'a syn::Attribute> + 'ret {
    attrs.iter().filter(move |attr| path_eq_single(&attr.path, name))
}

/// Compares the given `path` with the one-segment string `value`.
#[inline]
#[must_use]
fn path_eq_single(path: &syn::Path, value: &str) -> bool {
    path.segments.len() == 1 && path.segments[0].ident == value
}

pub mod field {
    use sealed::sealed;

    use crate::field::Container;

    use super::{Dedup, Kind};

    pub trait TryApply<V, K: Kind + ?Sized, D: Dedup + ?Sized>:
        Container<V>
    {
        fn try_apply(&mut self, val: V) -> syn::Result<()>;
    }

    pub trait TryApplySelf<V, K: Kind + ?Sized, D: Dedup + ?Sized>:
        TryApply<V, K, D>
    {
        fn try_apply_self(&mut self, another: Self) -> syn::Result<()>;
    }

    mod option {
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

    #[sealed]
    pub trait TryMerge<V> {
        fn try_merge<K, D>(&mut self, val: V) -> syn::Result<()>
        where
            Self: TryApply<V, K, D>,
            K: Kind + ?Sized,
            D: Dedup + ?Sized;

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
    use sealed::sealed;

    #[sealed]
    pub trait Kind {}

    #[sealed]
    pub trait Single {}

    pub enum Ident {}
    #[sealed]
    impl Kind for Ident {}
    #[sealed]
    impl Single for Ident {}

    pub enum Nested {}
    #[sealed]
    impl Kind for Nested {}
    #[sealed]
    impl Single for Nested {}

    pub enum Value {}
    #[sealed]
    impl Kind for Value {}
    #[sealed]
    impl Single for Value {}

    pub enum Map {}
    #[sealed]
    impl Kind for Map {}
}

pub mod dedup {
    use sealed::sealed;

    #[sealed]
    pub trait Dedup {}

    pub enum Unique {}
    #[sealed]
    impl Dedup for Unique {}

    pub enum First {}
    #[sealed]
    impl Dedup for First {}

    pub enum Last {}
    #[sealed]
    impl Dedup for Last {}
}

pub mod validate {
    use std::{
        collections::{HashMap, HashSet},
        hash::{BuildHasher, Hash},
    };

    use sealed::sealed;

    use crate::Required;

    pub use self::rule::Rule;

    pub trait Validate<R: Rule + ?Sized> {
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

    #[sealed]
    pub trait IsValid {
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
        use sealed::sealed;

        #[sealed]
        pub trait Rule {}

        pub enum Provided {}
        #[sealed]
        impl Rule for Provided {}
    }
}
