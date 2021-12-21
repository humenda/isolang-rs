#![cfg(unix)] // Avoid running on Windows: the generated code will use `\r\n` instead of `\n`

use std::collections::HashMap;
use std::env;
use std::fmt::Write as _;
use std::fs::{self, File};
use std::io::{BufRead, BufReader};
use std::path::Path;

// Taken from http://www-01.sil.org/iso639-3/download.asp
static ISO_TABLE_PATH: &str = "iso-639-3.tab";

// Local names of languages from https://github.com/bbqsrc/iso639-autonyms
static AUTONYMS_TABLE_PATH: &str = "iso639-autonyms.tsv";

/// Language data as extracted from `iso-639-3.tsv` and `iso-639-autonyms.tsv`.
///
/// This is a direct precursor to `isolang::LanguageData`, which has more comments.
struct LangCode {
    code_3: String,
    code_1: Option<String>,
    name_en: String,
    autonym: Option<String>,
}

/// Convert string into a equivalent version with the first character in upper case.
fn title(s: &str) -> String {
    s.chars()
        .next()
        .expect("Received empty string, cannot uppercase its first character")
        .to_uppercase()
        .chain(s[1..].chars())
        .collect::<String>()
}

// parse autonym table
fn read_autonyms_table() -> HashMap<String, Option<String>> {
    let r = BufReader::new(File::open(AUTONYMS_TABLE_PATH).expect(
        r"\
        Couldn't read autonyms table tsv. Make sure that this operation is run from \
        the crate source root and that this file actually exists.",
    ));

    r.lines()
        .skip(1)
        .map(|line| {
            let line = line.expect(
                "Couldn't read from autonyms table, please \
                    check that the file exists and is readable",
            );

            let cols = line.split('\t').collect::<Vec<&str>>();
            let three_letter: String = cols[0].into();
            let autonym: Option<String> = match cols[3].len() {
                0 => None,
                _ => Some(cols[3].into()),
            };

            (three_letter, autonym)
        })
        .collect()
}

/// parse ISO 6639-(3,1) table
fn read_iso_table() -> Vec<LangCode> {
    let autonyms_table = read_autonyms_table();

    let r = BufReader::new(File::open(ISO_TABLE_PATH).expect(
        r"\
        Couldn't read iso-639-3.tab. Make sure that this operation is run from \
        the crate source root and that this file actually exists.",
    ));
    r.lines()
        .skip(1)
        .filter_map(|line| line.ok())
        .map(|line| {
            let cols = line.split('\t').collect::<Vec<&str>>();
            let code_3: String = cols[0].into();
            let code_1: Option<String> = match cols[3].len() {
                2 => Some(cols[3].into()),
                _ => None,
            };
            let autonym = match autonyms_table.get(&code_3) {
                Some(Some(t)) => Some(t.to_owned()),
                _ => None,
            };

            // split language string into name and comment, if required
            let mut parts = cols[6].split('(');
            let name_en = parts.next().unwrap().trim_end();
            LangCode {
                code_3,
                code_1,
                name_en: name_en.into(),
                autonym,
            }
        })
        .collect()
}

/// write static array with (639-3, 639-1, english name, comment) entries
fn write_overview_table(out: &mut String, codes: &[LangCode]) {
    writeln!(
        out,
        "#[allow(clippy::type_complexity)]\npub(crate) static OVERVIEW: [LanguageData; {}] = [",
        codes.len()
    )
    .unwrap();

    for language in codes {
        writeln!(
            out,
            r#"    LanguageData {{
        code_3: {:?},
        code_1: {:?},
        #[cfg(feature = "english_names")]
        name_en: {:?},
        #[cfg(feature = "local_names")]
        autonym: {:?},
    }},"#,
            language.code_3.as_bytes(),
            language.code_1.as_ref().map(|s| s.as_bytes()),
            language.name_en,
            language.autonym,
        )
        .unwrap();
    }

    writeln!(out, "];").unwrap();
}

/// Write a mapping of codes from 639-1 -> Language::`639-3`
fn write_two_letter_to_enum(out: &mut String, codes: &[LangCode]) {
    write!(
        out,
        "pub(crate) static TWO_TO_THREE: phf::Map<&str, Language> = "
    )
    .unwrap();
    let mut map = phf_codegen::Map::new();
    for lang in codes.iter() {
        if let Some(ref two_letter) = lang.code_1 {
            map.entry(
                two_letter.as_str(),
                &format!("Language::{}", title(&lang.code_3)),
            );
        }
    }
    writeln!(out, "{};\n", map.build()).unwrap();
}

/// Write a mapping of codes from 639-3 -> Language::`639-3`
fn write_three_letter_to_enum(out: &mut String, codes: &[LangCode]) {
    write!(
        out,
        "pub(crate) static THREE_TO_THREE: phf::Map<&str, Language> = "
    )
    .unwrap();
    let mut map = phf_codegen::Map::new();
    for lang in codes.iter() {
        map.entry(
            lang.code_3.as_str(),
            &format!("Language::{}", title(&lang.code_3)),
        );
    }
    writeln!(out, "{};", map.build()).unwrap();
}

/// Check that the generated files are up to date
#[test]
fn generated_code_is_fresh() {
    let codes = read_iso_table();
    let mut src = String::with_capacity(1024 * 1024 + 1024 * 256); // Current size at 118k
    src.push_str(
        "/// This file is generated and should not be edited directly.\nuse super::LanguageData;\n\n",
    );

    // write overview table with all data
    write_overview_table(&mut src, &codes);

    // write enum with 639-3 codes (num is the index into the overview table)
    writeln!(
        &mut src,
        "#[derive(Clone, Copy, Hash, Eq, PartialEq, PartialOrd, Ord)]"
    )
    .unwrap();
    writeln!(&mut src, "pub enum Language {{").unwrap();
    for (num, lang) in codes.iter().enumerate() {
        writeln!(&mut src, "    #[doc(hidden)]").unwrap();
        writeln!(&mut src, "    {} = {},", title(&lang.code_3), num).unwrap();
    }
    writeln!(&mut src, "}}\n").unwrap();

    // write map 639-1 -> enum mapping
    write_two_letter_to_enum(&mut src, &codes);

    // write map 639-3 -> enum mapping
    write_three_letter_to_enum(&mut src, &codes);

    let path = Path::new(&env::var("CARGO_MANIFEST_DIR").unwrap()).join("src/isotable.rs");
    let old = fs::read_to_string(&path).unwrap();
    if old != src {
        fs::write(path, src).unwrap();
        panic!("generated code in the repository is outdated, updating...");
    }
}
