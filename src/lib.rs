//! ISO 639 language code crate
//!
//! When dealing with different language inputs and APIs, different standards are used to identify
//! a language. Converting between these in an automated way can be tedious. This crate provides an
//! enum which supports conversion from 639-1 and 639-3 and also into these formats, as well as
//! into English names.
//!
//! # Examples
//!
//! ```
//! use isolang::Language;
//!
//! assert_eq!(Language::from_639_1("de").unwrap().to_name(), "German");
//! assert_eq!(Language::from_639_3("spa").unwrap().to_639_1(), Some("es"));
//! ```

extern crate phf;

include!(concat!(env!("OUT_DIR"), "/isotable.rs"));

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
        OVERVIEW[*self as usize].0
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
        ///
        OVERVIEW[*self as usize].1
    }

    /// Get the English name of this language.
    ///
    /// This returns the English name of the language, as defined in the ISO 639 standard. It does
    /// not include additional comments, e.g. classification of a macrolanguage, etc.
    ///
    /// # Examples
    ///
    /// ```
    /// use isolang::Language;
    ///
    /// assert_eq!(Language::Spa.to_name(), "Spanish");
    /// // macro language
    /// assert_eq!(Language::Swa.to_name(), "Swahili");
    /// // individual language
    /// assert_eq!(Language::Swh.to_name(), "Swahili");
    /// ```
    pub fn to_name(&self) -> &'static str {
        OVERVIEW[*self as usize].2
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
}


#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
    }
}

