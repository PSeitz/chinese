mod jmdict;

use std::{collections::HashMap, fs, io::Write};

use pinyin_zhuyin::pinyin_to_zhuyin;
use prettify_pinyin::prettify;
use regex::{Captures, Regex};
use serde::{Deserialize, Serialize};
use tocfl::load_tocfl_dictionary;

use crate::jmdict::load_jmdict;

#[derive(Serialize, Deserialize, Debug, Default)]
struct FreqRow {
    text: String,
    count: u64, // count on its own
    count_per_million: f64, // count
                //log_count: f64,
}

#[derive(Serialize, Deserialize, Debug, Default)]
struct FreqEntry {
    text: String,
    count_self: u64,                  // count on its own
    count_per_million_self: f64,      // count_per_million on its own
    count_in_others: u64,             // how often does this part occur in others
    count_per_million_in_others: f64, // how often does this part occur in others
}
impl From<FreqRow> for FreqEntry {
    fn from(value: FreqRow) -> Self {
        Self {
            text: value.text,
            count_self: value.count,
            count_per_million_self: value.count_per_million,
            count_per_million_in_others: 0.0,
            count_in_others: 0,
        }
    }
}

struct Commonnness {
    char_freq: HashMap<String, FreqEntry>,
    word_freq: HashMap<String, FreqEntry>,
}

fn get_commonness() -> Commonnness {
    let mut char_freq = parse_commonness("../ch_freq/char_freq.json");
    let mut word_freq = parse_commonness("../ch_freq/word_freq.json");

    // Now we add the word parts to the chars, although single chars may be not that common
    // e.g. 午 on its own is uncommon, but 下午 [xiawu] is quite common
    for (word, v) in word_freq.iter_mut() {
        for cha in word.chars() {
            let entry = char_freq
                .entry(cha.to_string())
                .or_insert_with(Default::default);
            entry.count_in_others += v.count_self;
            entry.count_per_million_in_others += v.count_per_million_self;
        }
    }

    Commonnness {
        char_freq,
        word_freq,
    }
}

impl Commonnness {
    fn get_freq<'a>(
        &'a self,
        trad: &'a str,
        simpl: &'a str,
    ) -> impl Iterator<Item = &FreqEntry> + 'a {
        [
            self.word_freq.get(simpl),
            self.word_freq.get(trad),
            self.char_freq.get(simpl),
            self.char_freq.get(trad),
        ]
        .into_iter()
        .flatten()
    }

    fn get_count_per_million_self(&self, trad: &str, simpl: &str) -> f64 {
        self.get_freq(trad, simpl)
            .map(|el| el.count_per_million_self)
            .sum()
    }

    fn get_count_per_million_in_others(&self, trad: &str, simpl: &str) -> f64 {
        self.get_freq(trad, simpl)
            .map(|el| el.count_per_million_in_others)
            .sum()
    }

    fn get_count_self(&self, trad: &str, simpl: &str) -> u64 {
        self.get_freq(trad, simpl)
            .map(|el| el.count_self)
            .sum::<u64>()
    }

    fn get_count_in_others(&self, trad: &str, simpl: &str) -> u64 {
        self.get_freq(trad, simpl)
            .map(|el| el.count_in_others)
            .sum::<u64>()
    }
}

/// The subletex provides just hanzi without the pinyin.
/// This is of limited usage since we need the pinyin to differentiate between different meanings
/// (some of which are less commonn) of one Hanzi
fn parse_commonness(path: &str) -> HashMap<String, FreqEntry> {
    let data = std::fs::read_to_string(path).unwrap();
    data.lines()
        .map(|line| {
            let freq: FreqRow = serde_json::from_str(line).unwrap();
            //dbg!(&freq);
            (
                freq.text.to_string(),
                freq.into(), //(freq.log_count + 1.1, freq.count_per_million),
            )
        })
        .collect()
}

#[derive(Debug, serde::Deserialize)]
#[allow(dead_code)]
struct TocflEntry {
    #[serde(rename = "Word")]
    /// E.g. 台灣
    word: String,
    #[serde(rename = "Pinyin")]
    /// E.g. táiwān
    pinyin: String,
    #[serde(rename = "OtherPinyin")]
    other_pinyin: String,
    #[serde(rename = "Level")]
    level: u32,
    #[serde(rename = "First Translation")]
    first_translation: String,
    #[serde(rename = "Other Translation")]
    other_translations: Option<String>,
}

// Key is (Hanzi, Pinyin), to be unambigious
fn get_tocfl_levels() -> HashMap<(String, String), TocflEntry> {
    let mut tocfl_map = HashMap::new();
    let file = std::fs::read_to_string("./tocfl.csv").unwrap();

    let mut rdr = csv::Reader::from_reader(file.as_bytes());
    for result in rdr.deserialize() {
        let record: TocflEntry = result.unwrap();
        tocfl_map.insert((record.word.to_string(), record.pinyin.to_string()), record);
    }
    //dbg!(&tocfl_map.get("家"));

    tocfl_map
}

#[derive(Debug, Default)]
struct Radicals {
    traditional_to_radicals: HashMap<String, Vec<Vec<String>>>,
    simplified_to_radicals: HashMap<String, Vec<Vec<String>>>,
}

// Source: https://github.com/kfcd/chaizi
// LICENSE: https://github.com/kfcd/chaizi/blob/master/LICENSE
fn get_character_radicals() -> Radicals {
    let mut radicals = Radicals::default();

    let add_to_map = |file: String, map: &mut HashMap<String, Vec<Vec<String>>>| {
        for line in file.lines() {
            let line_parts: Vec<&str> = line.split("\t").collect();
            let kanji = line_parts[0];
            let radicals: Vec<Vec<String>> = line_parts[1..]
                .iter()
                .map(|el| el.split_whitespace().map(|el| el.to_string()).collect())
                .collect();
            map.insert(kanji.to_string(), radicals);
        }
    };
    let file = std::fs::read_to_string("./traditional_character_radicals.txt").unwrap();
    add_to_map(file, &mut radicals.traditional_to_radicals);

    let file = std::fs::read_to_string("./simplified_character_radicals.txt").unwrap();
    add_to_map(file, &mut radicals.simplified_to_radicals);

    //dbg!(&tocfl_map.traditional_to_radicals.get("家"));

    radicals
}

fn normalize_definitions(definitions: &mut Vec<String>) -> Option<String> {
    let taiwan_pr = Regex::new(r"Taiwan pr. \[(.*?)\]").unwrap();
    // pinyin regex
    let re = Regex::new(r"\[(.*?)\]").unwrap();

    let mut pinyin_taiwan = None;
    // Find alternative pinyin writings
    // "(Taiwan pr. [han4])"
    for text in definitions.iter() {
        for cap in taiwan_pr.captures_iter(text) {
            pinyin_taiwan = Some(cap[1].to_string());
        }
    }

    // Replace all pinyin with pretty
    for text in definitions.iter_mut() {
        *text = re
            .replace_all(text, |caps: &Captures| {
                let orig = &caps[1];
                let pretty = prettify(caps[1].to_string());
                if orig != pretty {
                    format!("[{}]", pretty)
                } else {
                    format!("[{}]", orig)
                }
            })
            .to_string();
    }

    // Flatten semicolon separated definitions
    let mut new_definitions = Vec::new();
    for text in definitions.iter() {
        for sub_def in text.split(";") {
            new_definitions.push(sub_def.trim().to_string());
        }
    }
    *definitions = new_definitions;

    pinyin_taiwan
}

fn main() {
    //let jmdict = load_jmdict("../../../japanese-dictionary/jmdict.json");

    let kanji_dict: KanjiDict =
        serde_json::from_str(&fs::read_to_string("./kanji.json").unwrap()).unwrap();

    let tocfl_dict = load_tocfl_dictionary();
    let common_char = tocfl::compile_common_chars();

    //let commonness = get_commonness();
    let radicals = get_character_radicals();

    let mut out = std::fs::File::create("db.json").unwrap();
    let all = std::fs::read_to_string("../cedict_ts.u8").unwrap();
    for line in all.lines() {
        let parsed = cedict::parse_line(line);
        let e = match parsed {
            cedict::Line::Entry(e) => e,
            cedict::Line::Comment(_) | cedict::Line::Metadata(_, _) | cedict::Line::Empty => {
                continue;
            }
            cedict::Line::Incorrect => {
                panic!("Incorrect line {}", line)
            }
        };

        let mut definitions = e.definitions().map(ToString::to_string).collect::<Vec<_>>();
        let pinyin_taiwan = normalize_definitions(&mut definitions);

        let pinyin_ws_tone_number = e.pinyin().to_string();
        let pinyin_pretty = prettify(e.pinyin().to_string());

        let zhuyin = pinyin_pretty
            .split_whitespace()
            .map(|pinyin_component| {
                if let Some(zhuyin) = pinyin_to_zhuyin(&pinyin_component) {
                    zhuyin
                } else {
                    pinyin_component.to_string()
                }
            })
            .collect::<Vec<String>>()
            .join(" ");

        let gen_pinyin_variations = |pinyin_with_ws_and_tone_numbers: &str| {
            vec![
                // jia1 li2
                pinyin_with_ws_and_tone_numbers.to_string(),
                // jiali
                pinyin_with_ws_and_tone_numbers.remove_whitespace(),
                // jia li
                pinyin_with_ws_and_tone_numbers.remove_numbers(),
                // jiali
                pinyin_with_ws_and_tone_numbers
                    .remove_numbers()
                    .remove_whitespace(),
                // jiā lǐ
                prettify(pinyin_with_ws_and_tone_numbers.to_string()),
                // jiālǐ
                prettify(pinyin_with_ws_and_tone_numbers.to_string()).remove_whitespace(),
            ]
        };

        let mut pinyin_search = gen_pinyin_variations(&pinyin_ws_tone_number);
        if let Some(pinyin_taiwan) = pinyin_taiwan.as_ref() {
            pinyin_search.extend_from_slice(&gen_pinyin_variations(&pinyin_taiwan));
        }

        let simplified = e.simplified().to_string();
        let traditional = e.traditional().to_string();

        let tocfl_entry = tocfl_dict.get_entry(
            &e.traditional(),
            &pinyin_taiwan
                .as_ref()
                .unwrap_or_else(|| &pinyin_ws_tone_number),
        );

        let tocfl_level = tocfl_entry.map(|entry| entry.tocfl_level);

        let mut count_per_million_written = tocfl_entry
            .map(|entry| entry.written_per_million)
            .unwrap_or(0);
        let count_per_million_spoken = tocfl_entry
            .map(|entry| entry.spoken_per_million)
            .unwrap_or(0);

        let count_per_million_in_others = *common_char
            .get_entry(&e.traditional(), e.pinyin())
            .unwrap_or(&0);
        if e.traditional() == "分" {
            dbg!(count_per_million_in_others);
            dbg!(count_per_million_written);
            dbg!(e.pinyin());
        }

        let kanji_char = kanji_hanzi_converter::convert_to_japanese_kanji(e.traditional());
        let kanji = kanji_dict.get(kanji_char.as_str()).map(|k| k.clone());

        let mut commonness_boost = (count_per_million_spoken as f64
            + count_per_million_written as f64)
            .sqrt()
            .max(4.0)
            / 4.0;
        assert!(!commonness_boost.is_nan());
        //assert!(!count_per_million_written.is_nan());

        let is_variant_entry = e.definitions().all(|def| def.contains("variant"));
        if is_variant_entry {
            count_per_million_written = 0;
            commonness_boost = 1.0;
        }

        let mut tags = Vec::new();
        if count_per_million_written > 150 {
            // top 1000
            tags.push("#common".to_string());
            tags.push("#common_spoken".to_string());
        }
        if count_per_million_spoken > 150 {
            // top 1000
            tags.push("#common".to_string());
            tags.push("#common_written".to_string());
        }

        if count_per_million_spoken > 450 {
            // top 300
            tags.push("#verycommon".to_string());
        }

        if count_per_million_in_others > 550 {
            tags.push("#commonchar".to_string());
        }

        if let Some(level) = tocfl_level {
            tags.push("#TOCFL".to_string());
            tags.push(format!("#TOCFL{}", level));
        }

        if let Some(kanji) = kanji.as_ref() {
            if let Some(level) = kanji.wk_level {
                tags.push("#WK".to_string());
                tags.push(format!("#WaniKaniLevel{}", level));
            }
        }

        let simplified = simplified.to_string();
        let traditional = traditional.to_string();

        let simplified_radicals = radicals
            .simplified_to_radicals
            .get(&simplified)
            .map(ToOwned::to_owned)
            .unwrap_or_default();
        let traditional_radicals = radicals
            .traditional_to_radicals
            .get(&traditional)
            .map(ToOwned::to_owned)
            .unwrap_or_default();

        let entry = Entry {
            simplified_radicals,
            traditional_radicals,
            simplified,
            traditional: traditional.to_string(),
            pinyin: e.pinyin().to_string(),
            pinyin_taiwan,
            pinyin_search: filter_duplicates(pinyin_search),
            zhuyin,
            pinyin_pretty,
            tocfl_level,
            meanings: definitions,
            commonness_boost,
            count_per_million_written,
            count_per_million_spoken,
            count_per_million_in_others,
            tags: filter_duplicates(tags),
            kanji,
        };
        out.write_all(serde_json::to_string(&entry).unwrap().as_bytes())
            .unwrap();
        out.write_all(b"\n").unwrap();
    }
    println!("Hello, world!");
}

trait RemoveWhiteSpace {
    fn remove_whitespace(self) -> String;
    fn remove_numbers(self) -> String;
}
impl RemoveWhiteSpace for String {
    fn remove_whitespace(self) -> String {
        self.chars()
            .filter(|el| !el.is_whitespace())
            .collect::<String>()
    }
    fn remove_numbers(self) -> String {
        self.chars()
            .filter(|el| !el.is_numeric())
            .collect::<String>()
    }
}
impl RemoveWhiteSpace for &str {
    fn remove_whitespace(self) -> String {
        self.chars()
            .filter(|el| !el.is_whitespace())
            .collect::<String>()
    }
    fn remove_numbers(self) -> String {
        self.chars()
            .filter(|el| !el.is_numeric())
            .collect::<String>()
    }
}

fn filter_duplicates(input: Vec<String>) -> Vec<String> {
    let mut result = Vec::new();
    let mut seen = std::collections::HashSet::new();

    for s in input {
        if seen.insert(s.clone()) {
            result.push(s);
        }
    }

    result
}

#[derive(Serialize)]
struct Entry {
    simplified: String,
    traditional: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    simplified_radicals: Vec<Vec<String>>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    traditional_radicals: Vec<Vec<String>>,
    pinyin: String,
    // Taiwanese pinyin with tone numbers
    pinyin_taiwan: Option<String>,
    // different pinyin variants for search. this could be covered by
    // tokenization but that's simpler
    pinyin_search: Vec<String>,
    zhuyin: String,
    pinyin_pretty: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    tocfl_level: Option<u32>,
    meanings: Vec<String>,
    tags: Vec<String>,
    commonness_boost: f64,
    count_per_million_written: u64,
    count_per_million_spoken: u64,
    count_per_million_in_others: u64,
    kanji: Option<KanjiCharacter>,
}

type KanjiDict = HashMap<String, KanjiCharacter>;

#[derive(Serialize, Deserialize, Clone)]
struct KanjiCharacter {
    strokes: u32,
    grade: Option<u32>,
    freq: Option<u32>,
    jlpt_old: Option<u32>,
    jlpt_new: Option<u32>,
    meanings: Vec<String>,
    readings_on: Vec<String>,
    readings_kun: Vec<String>,
    wk_level: Option<u32>,
    wk_meanings: Option<Vec<String>>,
    wk_readings_on: Option<Vec<String>>,
    wk_readings_kun: Option<Vec<String>>,
    wk_radicals: Option<Vec<String>>,
}
