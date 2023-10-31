ISO 639 language codes
=======================

[![Build Status](https://github.com/humenda/isolang-rs/workflows/CI/badge.svg)](https://github.com/humenda/isolang-rs/actions?query=workflow%3ACI)
[![Crates.io](https://img.shields.io/crates/v/isolang)](https://crates.io/crates/isolang)
[![Documentation](https://img.shields.io/docsrs/isolang)](https://docs.rs/isolang)
[![Licence: Apache 2.0](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](LICENCE.md)

Introduction
------------

When dealing with different language inputs and APIs, different standards are used to identify
a language. Converting between these in an automated way can be tedious. This crate provides an
enum which supports conversion from 639-1 and 639-3 and also into these formats, as well as
into English names or autonyms (local names).

This crate contains the ISO 639 table in statically embedded tables. This
increases binary size, but allows for very efficient look-up if performance
matters. If size is a concern, both and the English names and local names can be
enabled or disabled individually.

This crate is licensed under the Apache 2.0 license, please see LICENSE.md for
the details.

Usage
-----

`Cargo.toml`:

```toml
[dependencies]
isolang = "2.0"
```

Example
-------

```rust
use isolang::Language;

assert_eq!(Language::from_639_1("de").unwrap().to_name(), "German");
assert_eq!(Language::from_639_3("spa").unwrap().to_639_1(), Some("es"));
// undefined language (ISO code und)
assert_eq!(Language::default(), Language::Und);
```

```rust
use isolang::Language;
// `to_name()` is available if compiled with the `english_names` feature.
assert_eq!(Language::from_str("es").unwrap().to_name(), "Spanish");
assert_eq!(Language::from_str("spa").unwrap().to_name(), "Spanish");
// `from_str(lowercase_name)` is available if compiled with the `english_names` and `lowercase_names` features.
assert_eq!(Language::from_str("spanish").unwrap().to_name(), "Spanish");
// `from_str(local_name)` is available if compiled with the `english_names`, `lowercase_names` and `local_names` features.
assert_eq!(Language::from_str("español").unwrap().to_name(), "Spanish");
```

Serde support
-------------

This crate also supports serializing the `Language` enum. To enable this please
add the following lines to your `Cargo.toml` (instead of the above code):

```toml
[dependencies.isolang]
features = ["serde"]
version = "2.0"
```

Data Source
-----------

The data is downloaded from
<https://iso639-3.sil.org/code_tables/download_tables>,
or, alternatively from
<https://www.iana.org/assignments/language-subtag-registry/language-subtag-registry>.
