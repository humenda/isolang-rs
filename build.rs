extern crate phf_codegen;

use std::collections::HashSet;
use std::env;
use std::iter::FromIterator;
use std::fs::File;
use std::io::{BufReader, BufRead, BufWriter, Write};
use std::path::Path;

// Taken from http://www-01.sil.org/iso639-3/download.asp

static ISO_TABLE_PATH: &'static str = "iso-639-3.tab";
// title first character
fn title(s: &str) -> String {
    let mut v: Vec<char> = s.chars().collect();
    v[0] = v[0].to_uppercase().nth(0).unwrap();
    v.into_iter().collect::<String>()
}

fn read_iso_table() -> Vec<(String, String, String)> {
    let r = BufReader::new(File::open(ISO_TABLE_PATH).expect(r"\
        Couldn't read iso-639-3.tab. Make sure that his operation is run from \
        the crate source root and that this file actually exists."));
    r.lines().skip(1).map(|line| {
        let line = line.expect("Couldn't read from ISO 639 table, please check \
            that the file exists and is readable");
        let cols = line.split("\t").collect::<Vec<&str>>();
        (cols[0].into(), cols[3].into(), cols[6].into())
    }).collect()
}

fn main() {
    let path = Path::new(&env::var("OUT_DIR").unwrap()).join("isotable.rs");
    let codes = read_iso_table();

    { // make output file live shorter than codes
        let mut file = BufWriter::new(File::create(&path).expect(r"Couldn't \
                write to output directory, compilation impossible"));

        // write static array with (639-3, 639-1, English word) entries
        write!(&mut file, "pub enum Language {{");
        for (num, &(ref id, _, _)) in codes.iter().enumerate() {
            writeln!(&mut file, "    {} = {},", title(id), num).unwrap();
        }
        writeln!(&mut file, "}}\n\n").unwrap();

        // write map with English language name -> list of 639-3 codes
        let mut map = phf_codegen::Map::new();
        write!(&mut file, "static ENGLISH2IDX: phf::Map<&'static str, usize> = ").unwrap();
        for (num, &(_, _, ref engl)) in codes.iter().enumerate() {
            map.entry(engl.clone(), &num.to_string());
        }
        map.build(&mut file).unwrap();
        write!(&mut file, ";\n").unwrap();


        /* ToDo: solve issue of doubled ID's
        // write 639-1 to 639-3 mapping
        write!(&mut file, "static TWO_TO_THREE: phf::Map<&'static str, &'static str> = ").unwrap();
        for &(ref id, ref short, ref english) in codes.iter() {
            map.entry(short.clone(), id.clone().as_str());
        }
        map.build(&mut file).unwrap();
        write!(&mut file, ";\n").unwrap();
        */
    }
}

