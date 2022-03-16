#![cfg(unix)] // Avoid running on Windows: the generated code will use `\r\n` instead of `\n`

use std::collections::HashMap;
use std::io::{BufWriter, Write};
use std::fmt::Write as _;
use std::path::Path;
use std::process::{Command, Stdio};
use std::{env, fs};

// Taken from http://www-01.sil.org/iso639-3/download.asp
static ISO_TABLE_PATH: &str = "iso-639-3.tab";

// Local names of languages from https://github.com/bbqsrc/iso639-autonyms
static AUTONYMS_TABLE_PATH: &str = "iso639-autonyms.tsv";

/// Language data as extracted from `iso-639-3.tsv` and `iso-639-autonyms.tsv`.
struct LangCode<'a> {
    code_3: &'a str,
    code_1: Option<&'a str>,
    name_en: &'a str,
    autonym: Option<&'a str>,
}

struct Title<'a>(&'a str);

impl<'a> std::fmt::Display for Title<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut iter = self.0.chars();
        if let Some(c) = iter.next() {
            f.write_fmt(format_args!("{}", c.to_uppercase()))?;
        }
        f.write_str(iter.as_str())
    }
}

// parse autonym table
fn read_autonyms_table(table: &str) -> HashMap<&str, Option<&str>> {
    table
        .lines()
        .skip(1)
        .map(|line| {
            let mut cols = line.split('\t');
            let three_letter = cols.next().unwrap();
            (three_letter, cols.nth(2).filter(|s| !s.is_empty()))
        })
        .collect()
}

/// Parse ISO 6639-(3,1) table.
fn read_iso_table<'a>(iso_table: &'a str, autonyms_table: &'a str) -> Vec<LangCode<'a>> {
    let autonyms_table = read_autonyms_table(autonyms_table);
    iso_table
        .lines()
        .skip(1)
        .map(|line| {
            let mut cols = line.split('\t');
            let code_3 = cols.next().unwrap();
            let code_1 = cols.nth(2).filter(|s| s.len() == 2);
            let autonym = match autonyms_table.get(code_3) {
                Some(Some(t)) => Some(*t),
                _ => None,
            };

            // split language string into name and comment, if required
            let mut parts = cols.nth(2).unwrap().split('(');
            let name_en = parts.next().unwrap().trim_end();
            LangCode {
                code_3,
                code_1,
                name_en,
                autonym,
            }
        })
        .collect()
}

/// Write static array with (639-3, 639-1, english name, comment) entries.
fn write_overview_table(out: &mut String, codes: &[LangCode]) {
    writeln!(
        out,
        "#[allow(clippy::type_complexity)]\npub(crate) const OVERVIEW: [LanguageData; {}] = [",
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

/// Write a mapping of codes from 639-1 -> Language::`639-3`.
fn write_two_letter_to_enum(out: &mut String, codes: &[LangCode]) {
    write!(
        out,
        "pub(crate) const TWO_TO_THREE: phf::Map<&str, Language> = "
    )
    .unwrap();
    let mut map = phf_codegen::Map::new();
    for lang in codes.iter() {
        if let Some(ref two_letter) = lang.code_1 {
            map.entry(two_letter, &format!("Language::{}", Title(lang.code_3)));
        }
    }
    writeln!(out, "{};\n", map.build()).unwrap();
}

/// Write a mapping of codes from 639-3 -> Language::`639-3`.
fn write_three_letter_to_enum(out: &mut String, codes: &[LangCode]) {
    write!(
        out,
        "pub(crate) const THREE_TO_THREE: phf::Map<&str, Language> = "
    )
    .unwrap();
    let mut map = phf_codegen::Map::new();
    for lang in codes.iter() {
        map.entry(lang.code_3, &format!("Language::{}", Title(lang.code_3)));
    }
    writeln!(out, "{};", map.build()).unwrap();
}

/// Check that the generated files are up to date.
#[test]
fn generated_code_table_if_outdated() {
    let iso_table = fs::read_to_string(ISO_TABLE_PATH).expect(
        r"\
        Couldn't read iso-639-3.tab. Make sure that this operation is run from \
        the crate source root and that this file actually exists.",
    );
    let autonyms_table = fs::read_to_string(AUTONYMS_TABLE_PATH).expect(
        r"\
        Couldn't read autonyms table tsv. Make sure that this operation is run from \
        the crate source root and that this file actually exists.",
    );

    let codes = read_iso_table(&iso_table, &autonyms_table);
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
        writeln!(&mut src, "    {} = {},", Title(lang.code_3), num).unwrap();
    }
    writeln!(&mut src, "}}\n").unwrap();

    // write implementation for From<usize>
    writeln!(
        &mut src,
        "\nimpl Language {{\n"
    )
    .unwrap();
    writeln!(
        &mut src,
        "pub fn from_usize(u: usize) -> Option<Self> {{\n        match u {{"
    )
    .unwrap();
    for (num, lang) in codes.iter().enumerate() {
        writeln!(&mut src, "{} => Some(Language::{}),", num, Title(lang.code_3))
        .unwrap();
    }
    writeln!(&mut src, "    _ => None,").unwrap();

    writeln!(&mut src, "}} }} }}\n").unwrap();

    // write map 639-1 -> enum mapping
    write_two_letter_to_enum(&mut src, &codes);

    // write map 639-3 -> enum mapping
    write_three_letter_to_enum(&mut src, &codes);

    // compare old to new -- format new code first
    let child = Command::new("rustfmt")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .expect("Unable to format code, install rustfmt");
    {
        let mut childstdin = child.stdin.as_ref().unwrap();
        let mut writer = BufWriter::new(&mut childstdin);
        writer.write_all(src.as_bytes()).unwrap();
    }
    let output = child.wait_with_output().unwrap();
    if !output.status.success() {
        panic!("Unable to execute rustfmt");
    }

    let src = String::from_utf8(output.stdout).expect("Could not parse the generated source as UTF-8");

    let path = Path::new(&env::var("CARGO_MANIFEST_DIR").unwrap()).join("src/isotable.rs");
    let old = fs::read_to_string(&path).unwrap();
    // write new output and fail test to draw attention
    if old != src {
        fs::write(path.clone(), src).unwrap();
        Command::new("rustfmt")
            .arg(path)
            .spawn()
            .expect("Unable to format code, install rustfmt");

        panic!("generated code in the repository is outdated, updating...");
    }
}
