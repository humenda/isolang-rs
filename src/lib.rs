extern crate phf;

include!(concat!(env!("OUT_DIR"), "/isotable.rs"));

impl Language {
    pub fn to_639_3(&self) -> &'static str {
        OVERVIEW[*self as usize].0
    }

    pub fn to_639_1(&self) -> Option<&'static str> {
        OVERVIEW[*self as usize].1
    }

    pub fn to_name(&self) -> &'static str {
        OVERVIEW[*self as usize].2
    }

    pub fn from_639_1(code: &str) -> Option<Language> {
        if code.len() != 2 {
            return None;
        }
        TWO_TO_THREE.get(code).cloned()
    }

    pub fn from_639_3(code: &str) -> Option<Language> {
        if code.len() != 2 {
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

