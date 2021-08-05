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
//! ```

#[cfg(feature = "serde_serialize")]
use serde::{Serialize, Deserialize};

use std::str;

include!(concat!(env!("OUT_DIR"), "/isotable.rs"));

#[cfg_attr(feature = "serde_serialize", derive(Serialize, Deserialize))]
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
        unsafe { str::from_utf8_unchecked(&OVERVIEW[*self as usize].0) }
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
                .1
                .map(|ref s| str::from_utf8_unchecked(*s))
        }
    }

    #[cfg(feature = "english_names")]
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
    pub fn to_name(&self) -> &'static str {
        unsafe {
            // Is safe, see `to_639_3()` for more details
            str::from_utf8_unchecked(OVERVIEW[*self as usize].2)
        }
    }

    #[cfg(feature = "local_names")]
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
    pub fn to_autonym(&self) -> Option<&'static str> {
        unsafe {
            OVERVIEW[*self as usize]
                .3
                .map(|ref s| str::from_utf8_unchecked(*s))
        }
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
    /// fn main() {
    ///     assert!(Language::from_locale("de_DE.UTF-8") == Some(Language::Deu));
    /// }
    /// ```
    pub fn from_locale(locale: &str) -> Option<Language> {
        if locale.len() < 3 {
            return None;
        }
        // use first bit of locale (before the _) to detect the language
        locale
            .split('_')
            .next()
            .and_then(|langcode| Language::from_639_1(langcode))
    }
}

impl Default for Language {
    fn default() -> Self {
        Language::Und
    }
}

impl std::fmt::Debug for Language {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.to_639_3())
    }
}

impl std::fmt::Display for Language {
    #[cfg(feature = "local_names")]
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let autonym = match self.to_autonym() {
            Some(v) => v,
            None => "missing autonym",
        };

        write!(f, "{} ({})", self.to_name(), autonym)
    }

    #[cfg(all(not(feature = "local_names"), feature = "english_names"))]
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.to_name())
    }

    #[cfg(all(not(feature = "local_names"), not(feature = "english_names")))]
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.to_639_3())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(feature = "serde_serialize")]
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
        write!(t, "{}", Language::Eng).unwrap();
        if cfg!(feature = "local_names") {
            assert!("English (English)" == t);
        } else {
            assert!("English" == t);
        }

        let mut t = String::new();
        write!(t, "{:?}", Language::Eng).unwrap();
        assert!(String::from("eng") == t);
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
    #[cfg(feature = "serde_serialize")]
    fn test_serde() {
        assert!(serde_json::to_string(&Language::Deu).unwrap() == String::from("\"deu\""));
        assert!(serde_json::from_str::<Language>("\"deu\"").unwrap() == Language::Deu);
        assert!(serde_json::from_str::<Language>("\"fr\"").unwrap() == Language::Fra);

        assert!(serde_json::from_str::<Language>("\"foo\"").is_err());
    }

    #[test]
    fn test_ordering() {
        assert!(Language::Deu < Language::Fra);
        let fra = Language::Fra;
        assert!(fra <= Language::Fra);
    }
}
