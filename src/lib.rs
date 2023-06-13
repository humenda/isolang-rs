//! ISO 639 language codes
//!
//! When dealing with different language inputs and APIs, different standards are used to identify
//! a language. Converting between these in an automated way can be tedious. This crate provides an
//! enum which supports conversion from 639-1 and 639-3 and also into these formats, as well as
//! into their names. The English name can be retrieved using
//! [`Language::to_name()`](enum.Language.html#method.to_name) if compiled with the `english_names`
//! feature.
//! The autonyms (local names) can be retrieved using
//! [`to_autonym()`](enum.Language.html#method.to_autonym) if compiled with the `local_names`
//! feature.
//!
//! The language table is compiled into the library. While this increases the binary size, it means
//! that no additional time is wasted on program startup or on table access for allocating or
//! filling the map. It is hence suitable for retrieval of codes in constraint environments.
//!
//! # Examples
//!
//! ```
//! use isolang::Language;
//! #[cfg(feature = "english_names")]
//! assert_eq!(Language::from_639_1("de").unwrap().to_name(), "German");
//! #[cfg(feature = "local_names")]
//! assert_eq!(Language::from_639_1("de").unwrap().to_autonym(), Some("Deutsch"));
//!
//! assert_eq!(Language::from_639_3("spa").unwrap().to_639_1(), Some("es"));
//!
//! #[cfg(feature = "list_languages")]
//! {
//!     // Filter languages with a ISO 639-1 code
//!     let languages = isolang::languages();
//!     let languages_with_iso_639_1 = languages.filter(|language| language.to_639_1().is_some());
//!     for language in languages_with_iso_639_1 {
//!         assert_eq!(language.to_639_1().is_some(), true);
//!     }
//! }
//! ```

#[cfg(feature = "serde")]
mod serde_impl;

extern crate phf;

use std::{
    error::Error,
    fmt::{Debug, Display, Formatter},
    str::{self, FromStr},
};

/// Language data extracted from `iso-639-3.tab` and `iso639-autonyms.tsv`
///
/// Instances of this are generated in the `generated_code_is_fresh()` integration test,
/// which generates the code in `src/isotable.rs`.
struct LanguageData {
    /// The ISO-639-3 3-letter language code (column `Id` in `iso-639-3.tab`)
    code_3: [u8; 3],
    /// The ISO-639-1 2-letter language code, if available (column `Part1` in `iso-639-3.tab`)
    code_1: Option<[u8; 2]>,
    /// The language's name in English (column `Ref_Name` in `iso-639-3.tab`)
    ///
    /// The code generator removes any parenthesized suffix from the name.
    #[cfg(feature = "english_names")]
    name_en: &'static str,
    /// The language's lowercase name in English (column `Ref_Name` in `iso-639-3.tab`)
    #[cfg(feature = "lowercase_names")]
    name_en_lc: &'static str,
    /// The language's name in its own language (column `autonym` in `iso639-autonyms.tsv`)
    #[cfg(feature = "local_names")]
    autonym: Option<&'static str>,
}

#[rustfmt::skip]
mod isotable;
pub use isotable::Language;
use isotable::{OVERVIEW, THREE_TO_THREE, TWO_TO_THREE};

/// Get an iterator of all languages.
///
/// This will return an iterator over all the variants of the [`Language`](enum.Language.html) enum.
/// It is available if compiled with the `list_languages` feature.
///
/// # Examples
///
/// ```
/// let languages = isolang::languages();
///
/// // Display ISO 639-3 code of every language
/// for language in languages {
///     println!("{}", language.to_639_3());
/// }
///
/// // Filter languages with a ISO 639-1 code
/// let languages = isolang::languages();
/// let languages_with_iso_639_1 = languages.filter(|language| language.to_639_1().is_some());
/// for language in languages_with_iso_639_1 {
///     assert_eq!(language.to_639_1().is_some(), true);
/// }
/// ```
#[cfg(any(feature = "list_languages", test))]
pub fn languages() -> impl Iterator<Item = Language> {
    OVERVIEW.iter().enumerate().filter_map(|(idx, _)| Language::from_usize(idx))
}

impl Language {
    /// Create string representation of this Language as a ISO 639-3 code.
    ///
    /// This method will return the ISO 639-3 code, which consists of three letters.
    ///
    /// # Example
    ///
    /// ```
    /// use isolang::Language;
    ///
    /// assert_eq!(Language::Deu.to_639_3(), "deu");
    /// ```
    pub fn to_639_3(&self) -> &'static str {
        // It's safe to do so, we have written that by hand as UTF-8 into the binary and if you
        // haven't changed the binary, it's UTF-8
        unsafe { str::from_utf8_unchecked(&OVERVIEW[*self as usize].code_3) }
    }

    /// Create two-letter ISO 639-1 representation of the language.
    ///
    /// This will return a two-letter ISO 639-1 code, if it exists and None otherwise.
    /// ISO 639-1 codes are only used for the most common languages.
    ///
    /// # Example
    ///
    /// ```
    /// use isolang::Language;
    ///
    /// assert!(Language::Gha.to_639_1().is_none());
    /// ```
    pub fn to_639_1(&self) -> Option<&'static str> {
        unsafe {
            // Is safe, see `to_639_3()` for more details
            OVERVIEW[*self as usize]
                .code_1
                .as_ref()
                .map(|s| str::from_utf8_unchecked(s))
        }
    }

    /// Get the English name of this language.
    ///
    /// This returns the English name of the language, as defined in the ISO 639 standard. It does
    /// not include additional comments, e.g. classification of a macrolanguage, etc. It is
    /// available if compiled with the `english_names` feature.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use isolang::Language;
    ///
    /// assert_eq!(Language::Spa.to_name(), "Spanish");
    /// // macro language
    /// assert_eq!(Language::Swa.to_name(), "Swahili");
    /// // individual language
    /// assert_eq!(Language::Swh.to_name(), "Swahili");
    /// ```
    #[cfg(feature = "english_names")]
    pub fn to_name(&self) -> &'static str {
        OVERVIEW[*self as usize].name_en
    }

    /// Get the ISO code by its English name.
    ///
    /// This returns the ISO code by the given English name of the language string, as defined in
    /// the ISO 639 standard. It does not include additional comments, e.g. classification of a
    /// macrolanguage, etc. Only available if compiled with the `english_names` feature.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use isolang::Language;
    ///
    /// assert_eq!(Language::from_name("Spanish"), Some(Language::Spa));
    /// ```
    #[cfg(feature = "english_names")]
    pub fn from_name(engl_name: &str) -> Option<Self> {
        OVERVIEW
            .iter()
            .enumerate()
            .find(|(_, it)| it.name_en == engl_name)
            .and_then(|(idx, _)| Language::from_usize(idx))
    }

    /// Get the ISO code by its lowercase English name.
    ///
    /// This returns the ISO code by the given lowercase English name of the language string, as defined in
    /// the ISO 639 standard. It does not include additional comments, e.g. classification of a
    /// macrolanguage, etc. Only available if compiled with the `lowercase_names` feature.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use isolang::Language;
    /// let some_input_name = "spanish"; // maybe "Spanish"
    /// assert_eq!(Language::from_name_lowercase(&some_input_name.to_ascii_lowercase()), Some(Language::Spa));
    /// ```
    #[cfg(feature = "lowercase_names")]
    pub fn from_name_lowercase(engl_name: &str) -> Option<Self> {
        OVERVIEW
            .iter()
            .enumerate()
            .find(|(_, it)| it.name_en_lc == engl_name)
            .and_then(|(idx, _)| Language::from_usize(idx))
    }

    /// Get all matching ISO codes by a provided English name pattern.
    ///
    /// This returns the matching ISO codes for the provided matcher. The matcher matches all known
    /// English language names.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use isolang::Language;
    ///
    /// assert!(Language::match_names(|lang| lang.contains("Engl")).count() > 1);
    /// ```
    #[cfg(feature = "english_names")]
    pub fn match_names<F>(matcher: F) -> impl Iterator<Item = Self>
    where
        F: Fn(&str) -> bool + 'static,
    {
        OVERVIEW.iter().enumerate().filter_map(move |(idx, it)| {
            match matcher(it.name_en) {
                true => Language::from_usize(idx),
                false => None,
            }
        })
    }

    /// Get the autonym of this language
    ///
    /// This returns the native language name (if there is one available). This method is available
    /// if compiled with the `local_names` feature.
    /// The database for those names is found here https://github.com/bbqsrc/iso639-autonyms
    /// and it itself is a collection of several different datasets
    ///
    /// # Examples
    ///
    /// ```rust
    /// use isolang::Language;
    ///
    /// assert_eq!(Language::Bul.to_autonym(), Some("български"));
    /// assert_eq!(Language::Fra.to_autonym(), Some("français"));
    /// ```
    #[cfg(feature = "local_names")]
    pub fn to_autonym(&self) -> Option<&'static str> {
        OVERVIEW[*self as usize].autonym
    }

    /// Get the ISO code by its autonym (local language name).
    ///
    /// The result is `None` is the autonym wasn't found.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use isolang::Language;
    ///
    /// assert_eq!(Language::from_autonym("Deutsch"), Some(Language::Deu));
    /// ```
    #[cfg(feature = "local_names")]
    pub fn from_autonym(autonym: &str) -> Option<Self> {
        OVERVIEW
            .iter()
            .enumerate()
            .find(|(_, it)| it.autonym == Some(autonym))
            .and_then(|(idx, _)| Language::from_usize(idx))
    }

    /// Get all matching ISO codes by a provided autonym pattern.
    ///
    /// This returns the matching ISO codes for the provided matcher. It is evaluated against all
    /// known autonyms (local language names).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use isolang::Language;
    ///
    /// assert_eq!(Language::match_autonyms(|lang| lang.contains("Deutsch")).count(), 1);
    /// ```
    #[cfg(feature = "local_names")]
    pub fn match_autonyms<F>(matcher: F) -> impl Iterator<Item = Self>
    where
        F: Fn(&str) -> bool + 'static,
    {
        OVERVIEW.iter().enumerate().filter_map(move |(idx, it)| {
            it.autonym.and_then(|autonym| match matcher(autonym) {
                true => Language::from_usize(idx),
                false => None,
            })
        })
    }

    /// Create a Language instance rom a ISO 639-1 code.
    ///
    /// This will return a Language instance if the given string is a valid two-letter language
    /// code. For invalid inputs, None is returned.
    ///
    /// # Example
    ///
    /// ```
    /// use isolang::Language;
    ///
    /// assert!(Language::from_639_1("de").is_some());
    /// assert!(Language::from_639_1("…").is_none());
    /// ```
    pub fn from_639_1(code: &str) -> Option<Language> {
        if code.len() != 2 {
            return None;
        }
        TWO_TO_THREE.get(code).cloned()
    }

    /// Create a Language instance rom a ISO 639-3 code.
    ///
    /// This will return a Language instance if the given string is a valid three-letter language
    /// code. For invalid inputs, None is returned.
    ///
    /// # Example
    ///
    /// ```
    /// use isolang::Language;
    ///
    /// assert!(Language::from_639_3("dan").is_some());
    /// assert!(Language::from_639_1("…").is_none());
    /// ```
    pub fn from_639_3(code: &str) -> Option<Language> {
        if code.len() != 3 {
            return None;
        }
        THREE_TO_THREE.get(code).cloned()
    }

    /// Parse language from given locale
    ///
    /// This parses a language from a given locale string, as used by UNIX-alike and other systems.
    ///
    /// # Example
    ///
    /// ```
    /// use isolang::Language;
    ///
    /// assert!(Language::from_locale("de_DE.UTF-8") == Some(Language::Deu));
    /// ```
    pub fn from_locale(locale: &str) -> Option<Language> {
        if locale.len() < 3 {
            return None;
        }
        // use first bit of locale (before the _) to detect the language
        locale.split('_').next().and_then(Language::from_639_1)
    }
}

#[allow(clippy::derivable_impls)]
impl Default for Language {
    fn default() -> Self {
        Language::Und
    }
}

impl Debug for Language {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "{}", self.to_639_3())
    }
}

impl Display for Language {
    #[cfg(all(feature = "local_names", feature = "english_names"))]
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(
            f,
            "{} ({})",
            self.to_name(),
            self.to_autonym().unwrap_or("missing autonym")
        )
    }

    #[cfg(all(feature = "local_names", not(feature = "english_names")))]
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "{}", self.to_autonym().unwrap_or("missing autonym"))
    }

    #[cfg(all(not(feature = "local_names"), feature = "english_names"))]
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "{}", self.to_name())
    }

    #[cfg(all(not(feature = "local_names"), not(feature = "english_names")))]
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "{}", self.to_639_3())
    }
}

#[derive(Debug)]
pub struct ParseLanguageError(String);

impl Display for ParseLanguageError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "'{}' is not a valid ISO 639-1 or 639-3 code.", self.0)
    }
}

impl Error for ParseLanguageError {}

impl FromStr for Language {
    type Err = ParseLanguageError;

    fn from_str(s: &str) -> Result<Self, ParseLanguageError> {
        match Language::from_639_3(s).or_else(|| Language::from_639_1(s)) {
            Some(l) => Ok(l),
            None => Err(ParseLanguageError(s.to_owned())),
        }
    }

    #[cfg(feature = "lowercase_names")]
    fn from_str(s: &str) -> Result<Self, ParseLanguageError> {
        match Language::from_639_3(s)
            .or_else(|| Language::from_639_1(s))
            .or_else(|| Language::from_name_lowercase(s))
        {
            Some(l) => Ok(l),
            None => Err(ParseLanguageError(s.to_owned())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(feature = "serde")]
    extern crate serde_json;
    use std::fmt::Write;

    #[test]
    fn invalid_locale_gives_none() {
        assert!(Language::from_locale("foo").is_none());
        assert!(Language::from_locale("deu_DEU.UTF-8").is_none());
        assert!(Language::from_locale("___").is_none());
        assert!(Language::from_locale("ää_öö.UTF-8").is_none());
    }

    #[test]
    fn test_valid_locales_are_correctly_decoded() {
        assert_eq!(Language::from_locale("de_DE.UTF-8"), Some(Language::Deu));
        assert_eq!(Language::from_locale("en_GB.UTF-8"), Some(Language::Eng));
    }

    #[test]
    fn test_std_fmt() {
        let mut t = String::new();
        write!(t, "{}", Language::Deu).unwrap();
        if cfg!(feature = "local_names") && cfg!(feature = "english_names") {
            assert_eq!(t, "German (Deutsch)");
        } else if cfg!(feature = "local_names") {
            assert_eq!(t, "Deutsch");
        } else if cfg!(feature = "english_names") {
            assert_eq!(t, "German");
        } else {
            assert_eq!(t, "deu");
        }

        let mut t = String::new();
        write!(t, "{:?}", Language::Deu).unwrap();
        assert_eq!(t, "deu");
    }

    #[test]
    #[cfg(feature = "local_names")]
    fn test_iso639_3_to_autonym() {
        assert_eq!(
            Language::from_639_3("bul").unwrap().to_autonym(),
            Some("български")
        );
        assert_eq!(
            Language::from_639_3("fra").unwrap().to_autonym(),
            Some("français")
        );
    }

    #[test]
    fn test_default() {
        assert_eq!(Language::default(), Language::Und);
    }

    #[test]
    #[cfg(feature = "serde")]
    fn test_serde() {
        fn to_json(code: &str) -> String {
            format!(r#""{code}""#)
        }

        fn test_deserialize(language: Language, code: &str) {
            assert_eq!(
                serde_json::from_str::<Language>(&to_json(code)).unwrap(),
                language
            );
            assert_eq!(
                serde_json::from_value::<Language>(serde_json::json!(code))
                    .unwrap(),
                language
            );
        }

        for language in languages() {
            assert_eq!(
                serde_json::to_string(&language).unwrap(),
                to_json(language.to_639_3())
            );

            test_deserialize(language, language.to_639_3());
            if let Some(code) = language.to_639_1() {
                test_deserialize(language, code)
            }

            assert_eq!(
                serde_json::from_str::<Language>(
                    &serde_json::to_string(&language).unwrap()
                )
                .unwrap(),
                language
            );
        }

        assert_eq!(
            serde_json::from_str::<Language>(&to_json("foo")).map_err(|e| e.to_string()),
            Err("unknown variant `foo`, expected `any valid ISO 639-1 or 639-3 code` at line 1 column 5".to_string())
        );
        assert_eq!(
            serde_json::from_str::<Language>("123").map_err(|e| e.to_string()),
            Err("invalid type: integer `123`, expected borrowed str or bytes at line 1 column 3".to_string())
        );
    }

    #[test]
    fn test_ordering() {
        assert!(Language::Deu < Language::Fra);
        let fra = Language::Fra;
        assert!(fra <= Language::Fra);
    }

    #[test]
    #[cfg(feature = "list_languages")]
    fn test_good_language_filtering() {
        let languages = languages();
        let languages_with_iso_639_1 =
            languages.filter(|language| language.to_639_1().is_some());
        for language in languages_with_iso_639_1 {
            assert!(language.to_639_1().is_some());
        }
    }

    #[test]
    #[cfg(feature = "list_languages")]
    fn test_wrong_language_filtering() {
        let languages = languages();
        let languages_with_iso_639_1 =
            languages.filter(|language| language.to_639_1().is_none());
        for language in languages_with_iso_639_1 {
            assert!(language.to_639_1().is_none());
        }
    }

    #[test]
    fn test_from_str() {
        assert_eq!(Language::from_str("deu").unwrap(), Language::Deu);
        assert_eq!(Language::from_str("fr").unwrap(), Language::Fra);
        if cfg!(feature = "lowercase_names") {
            assert_eq!(Language::from_str("english").unwrap(), Language::Eng);
        }
        assert!(Language::from_str("foo").is_err());
    }
}
