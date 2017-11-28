use ::*;

impl serde::ser::Serialize for Language {
    fn serialize<S>(&self, s: S) -> Result<S::Ok, S::Error>
            where S: serde::ser::Serializer {
        s.serialize_str(self.to_639_3())
    }
}

#[derive(Clone, Copy)]
struct LanguageVisitor;

impl<'a> serde::de::Visitor<'a> for LanguageVisitor {
    type Value = Language;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a borrowed str")
    }

    fn visit_borrowed_str<E>(self, v: &'a str) -> Result<Self::Value, E>
            where E: serde::de::Error {
        match Language::from_639_3(v) {
            Some(l) => Ok(l),
            None => Err(serde::de::Error::unknown_variant(
                v,
                &["Any valid ISO 639-1 Code."],
            )),
        }
    }

    fn visit_borrowed_bytes<E>(self, v: &'a [u8]) -> Result<Self::Value, E>
            where E: serde::de::Error {
        self.visit_borrowed_str(str::from_utf8(v).map_err(|_| {
            serde::de::Error::invalid_value(serde::de::Unexpected::Bytes(v), &self)
        })?)
    }
}

#[cfg(feature = "serde_serialize")]
impl<'de> serde::de::Deserialize<'de> for Language {
    fn deserialize<D: serde::de::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_str(LanguageVisitor)
    }
}
