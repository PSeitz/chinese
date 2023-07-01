use std::io::{BufRead, BufReader};

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
struct Kanji {
    text: String,
    ent_seq: String,
    commonness: Option<u32>,
    readings: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize)]
struct Kana {
    text: String,
    ent_seq: String,
    romaji: String,
    commonness: Option<u32>,
}

#[derive(Debug, Deserialize, Serialize)]
struct Meanings {
    eng: Vec<String>,
    ger: Option<Vec<German>>,
}

#[derive(Debug, Deserialize, Serialize)]
struct German {
    text: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct JapaneseWord {
    ent_seq: String,
    commonness: Option<u32>,
    pos: Vec<Option<String>>,
    misc: Vec<String>,
    kanji: Vec<Kanji>,
    kana: Vec<Kana>,
    meanings: Meanings,
    #[serde(default)]
    useKana: bool,
}

pub fn load_jmdict(path: &str) -> Vec<JapaneseWord> {
    let file = std::fs::File::open(path).unwrap();
    let reader = BufReader::new(file);

    let mut kanji_count = 0;
    let entries: Vec<_> = reader
        .lines()
        .map(|line| {
            let line = line.unwrap();
            //println!("{}", &line);

            let word: JapaneseWord = serde_json::from_str(&line).unwrap();
            kanji_count += word.kanji.len();
            word
        })
        .collect();
    //dbg!(kanji_count);
    //dbg!(entries.len());
    //dbg!(has_duplicate_kanji(&entries));

    entries
}
