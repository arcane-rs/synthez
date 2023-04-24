`synthez` changelog
===================

All user visible changes to this project will be documented in this file. This project uses [Semantic Versioning 2.0.0].




## [0.3.1] · 2023-03-24
[0.3.1]: /../../tree/v0.3.1

[Diff](/../../compare/v0.3.0...v0.3.1)

### Changed

- Upgraded [`sealed`] to 0.5 version to fully get rid of [`syn`] 1.0. ([147baf04])

[147baf04]: /../../commit/147baf047ff840776346048afdafe77ccb94486b




## [0.3.0] · 2023-03-21
[0.3.0]: /../../tree/v0.3.0

[Diff](/../../compare/v0.2.0...v0.3.0)

### BC Breaks

- Set MSRV to [1.62.0](https://blog.rust-lang.org/2022/06/30/Rust-1.62.0.html). ([7f0b77e0])
- Upgrade [`syn`] to 2.0 version. ([90159de5])

[7f0b77e0]: /../../commit/7f0b77e0842edd7ecd18c91ec1e1b218711cc230
[90159de5]: /../../commit/90159de521e71c0d0cffbdb38dcb21e9ffe84227




## [0.2.0] · 2021-10-27
[0.2.0]: /../../tree/v0.2.0

[Diff](/../../compare/v0.1.3...v0.2.0)

### BC Breaks

- Set MSRV to [1.56.0](https://blog.rust-lang.org/2021/10/21/Rust-1.56.0.html). ([3e6d0846])

### Fixed

- Broken links to [`syn`] declarations in Rust documentation. ([3e6d0846])

[3e6d0846]: /../../commit/3e6d08464ef66b1e3ca47a0afda1175e6ce15a95




## [0.1.3] · 2021-08-27
[0.1.3]: /../../tree/v0.1.3

[Diff](/../../compare/v0.1.2...v0.1.3)

### Added

- `Required::into_inner()` method for unwrapping this container. ([6fbda59c])

### Fixed

- Unintentionally exposed `Required::is_present()` and `Required::replace()` methods. ([6fbda59c])

[6fbda59c]: /../../commit/6fbda59c5940effd32e66592602007dece082fcc




## [0.1.2] · 2021-08-25
[0.1.2]: /../../tree/v0.1.2

[Diff](/../../compare/v0.1.1...v0.1.2)

### Fixed

- Non-deterministic error messages. ([#2])

[#2]: /../../pull/2




## [0.1.1] · 2021-08-13
[0.1.1]: /../../tree/v0.1.1

[Diff](/../../compare/v0.1.0...v0.1.1)

### Fixed

- Raw identifiers (with `r#`) expanding as-is. ([#1])

[#1]: /../../pull/1




## [0.1.0] · 2021-06-25
[0.1.0]: /../../tree/v0.1.0

### Initially implemented 

- `ParseAttrs` trait and derive macro for parsing `syn::Attribute`s in declarative way.
- Primitive `ToTokens` derive macro supporting only `#[to_tokens(append(<method>))]` attribute.
- `parse:attr::doc()`/`parse:attr::doc_string()` helpers for convenient parsing normalized Rust doc comments and `#[doc]` attributes.
- `Spanning` wrapper for attaching `Span` to arbitrary types.
- `Required` container for denoting `ParseAttrs` fields required to be provided.
- `IntoSpan` coercion working for both `Span` and `Spanned` types at the same time.
- `has::Attrs` trait abstracting `syn` types which contain `syn::Attribute`s.
- Extensions:
    - `IdentExt` simplifying `syn::Ident` creation;
    - `DataExt` simplifying `syn::Data` fields usage;
    - `ParseBufferExt` providing batteries for parsing `syn::Ident` and `syn::punctuated`.




[`sealed`]: https://docs.rs/sealed
[`syn`]: https://docs.rs/syn

[Semantic Versioning 2.0.0]: https://semver.org
