extern crate phf_codegen;

use std::env;
use std::fs::File;
use std::io::{BufReader, BufRead, BufWriter, Write};
use std::path::Path;

// Taken from http://www-01.sil.org/iso639-3/download.asp
static ISO_TABLE_PATH: &'static str = "iso-639-3.tab";

/// This contains (639-3, 639-1, English name, comment)
type LangCodes = Vec<(String, Option<String>, String, Option<String>)>;


/// convert first character to upper case
fn title(s: &str) -> String {
    let mut v: Vec<char> = s.chars().collect();
    v[0] = v[0].to_uppercase().nth(0).unwrap();
    v.into_iter().collect::<String>()
}

/// parse ISO 6639-(3,1) table
fn read_iso_table() -> LangCodes {
    let r = BufReader::new(File::open(ISO_TABLE_PATH).expect(r"\
        Couldn't read iso-639-3.tab. Make sure that his operation is run from \
        the crate source root and that this file actually exists."));
    r.lines().skip(1).map(|line| {
        let line = line.expect("Couldn't read from ISO 639 table, please check \
            that the file exists and is readable");
        let cols = line.split("\t").collect::<Vec<&str>>();
        let two_letter: Option<String> = match cols[3].len() {
            2 => Some(cols[3].into()),
            _ => None
        };
        // split language string into name and comment, if required
        match cols[6].contains("(") {
            false => (cols[0].into(), two_letter, cols[6].into(), None),
            true => match cols[6].split(" (").collect::<Vec<&str>>() {
                ref m if m.len() != 2  => (cols[0].into(), two_letter, cols[6].into(), None),
                m => (cols[0].into(), two_letter, m[0].into(), Some(m[1].into())),
            }
        }
    }).collect()
}

/// write static array with (639-3, 639-1, English word, comment) entries
fn write_overview_table(file: &mut BufWriter<File>, codes: &LangCodes) {
    writeln!(file, "static OVERVIEW: [([u8; 3], Option<&'static [u8; 2]>, \
            &'static [u8], Option<&'static [u8]>); {}] = [", codes.len()).unwrap();
    for ref line in codes.iter() {
        write!(file, "    ({:?}, ", line.0.as_bytes()).unwrap();
        match line.1 {
            Some(ref val) => write!(file, "Some(&{:?}), ", val.as_bytes()).unwrap(),
            None => write!(file, "None, ").unwrap(),
        }
        write!(file, "&{:?}, ", line.2.as_bytes()).unwrap();
        match line.3 {
            Some(ref comment) => writeln!(file, "Some(&{:?})),", comment.as_bytes()).unwrap(),
            None => writeln!(file, "None),").unwrap(),
        };
    }
    write!(file, "];\n").unwrap();
}


/// Write a mapping of codes from 639-1 -> Language::`639-3`
fn write_two_letter_to_enum(file: &mut BufWriter<File>, codes: &LangCodes) {
    write!(file, "static TWO_TO_THREE: phf::Map<&'static str, Language> = ").unwrap();
    let mut map = phf_codegen::Map::new();
    for &(ref id, ref two_letter, _, _) in codes.iter() {
        if let &Some(ref two_letter) = two_letter {
            map.entry(two_letter.clone(), &format!("Language::{}", title(id)));
        }
    }
    map.build(file).unwrap();
    writeln!(file, ";").unwrap();
}

/// Write a mapping of codes from 639-3 -> Language::`639-3`
fn write_three_letter_to_enum(file: &mut BufWriter<File>, codes: &LangCodes) {
    write!(file, "static THREE_TO_THREE: phf::Map<&'static str, Language> = ").unwrap();
    let mut map = phf_codegen::Map::new();
    for &(ref id, _, _, _) in codes.iter() {
        map.entry(id.clone(), &format!("Language::{}", title(id)));
    }
    map.build(file).unwrap();
    writeln!(file, ";").unwrap();
}



fn main() {
    let path = Path::new(&env::var("OUT_DIR").unwrap()).join("isotable.rs");
    let codes = read_iso_table();

    { // make output file live shorter than codes
        let mut file = BufWriter::new(File::create(&path).expect(r"Couldn't \
                write to output directory, compilation impossible"));

        // write overview table with all data
        write_overview_table(&mut file, &codes);

        // write enum with 639-3 codes (num is the index into the overview table)
        writeln!(&mut file, "\n#[derive(Clone, Copy, Eq, PartialEq)]").unwrap();
        writeln!(&mut file, "pub enum Language {{").unwrap();
        for (num, &(ref id, _, _, _)) in codes.iter().enumerate() {
            writeln!(&mut file, "    {} = {},", title(id), num).unwrap();
        }
        writeln!(&mut file, "}}\n\n").unwrap();

        // write map 639-1 -> enum mapping
        write_two_letter_to_enum(&mut file, &codes);

        // write map 639-3 -> enum mapping
        write_three_letter_to_enum(&mut file, &codes);
    }
}

