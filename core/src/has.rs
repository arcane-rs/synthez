//! Abstraction over [`syn`] types containing something.

/// [`syn`] types containing [`syn::Attribute`]s.
pub trait Attrs {
    /// Returns contained [`syn::Attribute`]s.
    #[must_use]
    fn attrs(&self) -> &[syn::Attribute];
}

impl Attrs for Vec<syn::Attribute> {
    fn attrs(&self) -> &[syn::Attribute] {
        &*self
    }
}

/// Helper macro for implementing [`Attrs`] for the given type.
macro_rules! impl_attrs_for {
    ($( $ty:ty, )+) => {$(
        impl Attrs for $ty {
            fn attrs(&self) -> &[syn::Attribute] {
                &*self.attrs
            }
        }
    )+}
}

impl_attrs_for! {
    syn::BareFnArg,
    syn::ConstParam,
    syn::DeriveInput,
    syn::Field,
    syn::LifetimeDef,
    syn::TypeParam,
    syn::Variadic,
    syn::Variant,
}

#[cfg(feature = "full")]
/// Helper macro for implementing [`Attrs`] for the given type, conditioned by a
/// `full` Cargo feature.
macro_rules! impl_attrs_full_for {
    ($( $ty:ty, )+) => {$(
        #[cfg(feature = "full")]
        impl Attrs for $ty {
            fn attrs(&self) -> &[syn::Attribute] {
                &*self.attrs
            }
        }
    )+}
}

#[cfg(feature = "full")]
impl_attrs_full_for! {
    syn::Arm,
    syn::ExprArray,
    syn::ExprAssign,
    syn::ExprAssignOp,
    syn::ExprAsync,
    syn::ExprAwait,
    syn::ExprBinary,
    syn::ExprBlock,
    syn::ExprBox,
    syn::ExprBreak,
    syn::ExprCall,
    syn::ExprCast,
    syn::ExprClosure,
    syn::ExprContinue,
    syn::ExprField,
    syn::ExprForLoop,
    syn::ExprGroup,
    syn::ExprIf,
    syn::ExprIndex,
    syn::ExprLet,
    syn::ExprLit,
    syn::ExprLoop,
    syn::ExprMacro,
    syn::ExprMatch,
    syn::ExprMethodCall,
    syn::ExprParen,
    syn::ExprPath,
    syn::ExprRange,
    syn::ExprReference,
    syn::ExprRepeat,
    syn::ExprReturn,
    syn::ExprStruct,
    syn::ExprTry,
    syn::ExprTryBlock,
    syn::ExprTuple,
    syn::ExprType,
    syn::ExprUnary,
    syn::ExprUnsafe,
    syn::ExprWhile,
    syn::ExprYield,
    syn::FieldPat,
    syn::FieldValue,
    syn::File,
    syn::ForeignItemFn,
    syn::ForeignItemMacro,
    syn::ForeignItemStatic,
    syn::ForeignItemType,
    syn::ImplItemConst,
    syn::ImplItemMacro,
    syn::ImplItemMethod,
    syn::ImplItemType,
    syn::ItemConst,
    syn::ItemEnum,
    syn::ItemExternCrate,
    syn::ItemFn,
    syn::ItemForeignMod,
    syn::ItemImpl,
    syn::ItemMacro,
    syn::ItemMacro2,
    syn::ItemMod,
    syn::ItemStatic,
    syn::ItemStruct,
    syn::ItemTrait,
    syn::ItemTraitAlias,
    syn::ItemType,
    syn::ItemUnion,
    syn::ItemUse,
    syn::Local,
    syn::PatBox,
    syn::PatIdent,
    syn::PatLit,
    syn::PatMacro,
    syn::PatOr,
    syn::PatPath,
    syn::PatRange,
    syn::PatReference,
    syn::PatRest,
    syn::PatSlice,
    syn::PatStruct,
    syn::PatTuple,
    syn::PatTupleStruct,
    syn::PatType,
    syn::PatWild,
    syn::Receiver,
    syn::TraitItemConst,
    syn::TraitItemMacro,
    syn::TraitItemMethod,
    syn::TraitItemType,
}
