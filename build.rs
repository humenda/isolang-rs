extern crate phf_codegen;

use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::Path;

// Taken from http://www-01.sil.org/iso639-3/download.asp
static ISO_TABLE_PATH: &str = "iso-639-3.tab";

// Local names of languages from https://github.com/bbqsrc/iso639-autonyms
static AUTONYMS_TABLE_PATH: &str = "iso639-autonyms.tsv";

pub struct Language {
    english: String,
    local: Option<String>,
}

/// This contains (639-3, 639-1, Name, comment)
type LangCode = (String, Option<String>, Language, Option<String>);

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
            let three_letter: String = cols[0].into();
            let two_letter: Option<String> = match cols[3].len() {
                2 => Some(cols[3].into()),
                _ => None,
            };
            let autonym = &autonyms_table[&three_letter];
            // split language string into name and comment, if required
            if !cols[6].contains('(') {
                (
                    three_letter,
                    two_letter,
                    Language {
                        english: cols[6].into(),
                        local: autonym.to_owned(),
                    },
                    None,
                )
            } else {
                match cols[6].split(" (").collect::<Vec<&str>>() {
                    ref m if m.len() != 2 => (
                        three_letter,
                        two_letter,
                        Language {
                            english: cols[6].into(),
                            local: autonym.to_owned(),
                        },
                        None,
                    ),
                    m => (
                        three_letter,
                        two_letter,
                        Language {
                            english: m[0].into(),
                            local: autonym.to_owned(),
                        },
                        Some(m[1].into()),
                    ),
                }
            }
        })
        .collect()
}

/// write static array with (639-3, 639-1, english name, comment) entries
fn write_overview_table(file: &mut BufWriter<File>, codes: &[LangCode]) {
    if cfg!(feature = "local_names") {
        writeln!(
            file,
            "static OVERVIEW: [([u8; 3], Option<&[u8; 2]>, \
                &[u8], Option<&[u8]>, Option<&[u8]>); {}] = [",
            codes.len()
        )
        .unwrap();
    } else {
        writeln!(
            file,
            "static OVERVIEW: [([u8; 3], Option<&[u8; 2]>, \
                &[u8], Option<&[u8]>); {}] = [",
            codes.len()
        )
        .unwrap();
    }
    for ref language in codes.iter() {
        write!(file, "    ({:?}, ", language.0.as_bytes()).unwrap();
        match language.1 {
            Some(ref val) => write!(file, "Some(&{:?}), ", val.as_bytes()).unwrap(),
            None => write!(file, "None, ").unwrap(),
        }

        write!(file, "&{:?}, ", language.2.english.as_bytes()).unwrap();

        if cfg!(feature = "local_names") {
            match language.2.local {
                Some(ref val) => write!(file, "Some(&{:?}), ", val.as_bytes()).unwrap(),
                None => write!(file, "None, ").unwrap(),
            }
        }

        match language.3 {
            Some(ref comment) => writeln!(file, "Some(&{:?})),", comment.as_bytes()).unwrap(),
            None => writeln!(file, "None),").unwrap(),
        };
    }
    writeln!(file, "];").unwrap();
}

/// Write a mapping of codes from 639-1 -> Language::`639-3`
fn write_two_letter_to_enum(file: &mut BufWriter<File>, codes: &[LangCode]) {
    write!(
        file,
        "static TWO_TO_THREE: phf::Map<&str, Language> = "
    )
    .unwrap();
    let mut map = phf_codegen::Map::new();
    for &(ref id, ref two_letter, _, _) in codes.iter() {
        if let Some(ref two_letter) = two_letter {
            map.entry(two_letter.as_str(), &format!("Language::{}", title(id)));
        }
    }
    writeln!(file, "{};", map.build()).unwrap();
}

/// Write a mapping of codes from 639-3 -> Language::`639-3`
fn write_three_letter_to_enum(file: &mut BufWriter<File>, codes: &[LangCode]) {
    write!(
        file,
        "static THREE_TO_THREE: phf::Map<&str, Language> = "
    )
    .unwrap();
    let mut map = phf_codegen::Map::new();
    for &(ref id, _, _, _) in codes.iter() {
        map.entry(id.as_str(), &format!("Language::{}", title(id)));
    }
    writeln!(file, "{};", map.build()).unwrap();
}

fn main() {
    let path = Path::new(&env::var("OUT_DIR").unwrap()).join("isotable.rs");
    let codes = read_iso_table();

    {
        // make output file live shorter than codes
        let mut file = BufWriter::new(File::create(&path).expect(
            r"Couldn't \
                write to output directory, compilation impossible",
        ));

        // write overview table with all data
        write_overview_table(&mut file, &codes);

        // write enum with 639-3 codes (num is the index into the overview table)
        writeln!(
            &mut file,
            "#[derive(Clone, Copy, Hash, Eq, PartialEq, PartialOrd, Ord)]"
        )
        .unwrap();
        writeln!(&mut file, "pub enum Language {{").unwrap();
        for (num, &(ref id, _, _, _)) in codes.iter().enumerate() {
            writeln!(&mut file, "    #[doc(hidden)]").unwrap();
            writeln!(&mut file, "    {} = {},", title(id), num).unwrap();
        }
        writeln!(&mut file, "}}\n\n").unwrap();

        // write map 639-1 -> enum mapping
        write_two_letter_to_enum(&mut file, &codes);

        // write map 639-3 -> enum mapping
        write_three_letter_to_enum(&mut file, &codes);
    }
}
