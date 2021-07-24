# ISO 639 language codes

[![Build Status](https://travis-ci.org/humenda/isolang-rs.svg?branch=master)](https://travis-ci.org/humenda/isolang-rs)
[![Crates.io](https://img.shields.io/crates/v/isolang)](https://crates.io/crates/isolang)
[![Documentation](https://img.shields.io/docsrs/isolang)](https://docs.rs/isolang)
[![Licence](LICENCE.md)

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
isolang = "1.0"
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

Serde support
-------------

This crate also supports serializing the `Language` enum. To enable this please
add the following lines to your `Cargo.toml` (instead of the above code):

```toml
[dependencies.isolang]
features = ["serde_serialize"]
version = "1.0"
```
