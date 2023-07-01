use std::{collections::HashMap, path::PathBuf};

use measure_time::*;

//use serde_json::json;
//use tantivy::{
//collector::TopDocs,
//query::{BooleanQuery, Occur, Query, TermQuery},
//schema::{Field, FieldType, IndexRecordOption},
//DocId, Index, Score, SegmentReader, Term,
//};
use veloci::{
    error::VelociError,
    persistence::{self, Persistence},
    query_generator::generate_phrase_queries_simple,
    result::SearchResultWithDoc,
    search::{self, RequestBoostPart, RequestSearchPart, SearchRequest},
};

//pub const COMMON_CJK: [u32; 2] = [0x4E00, 0x9FFF];
fn is_chinese(cha: char) -> bool {
    matches!(cha as u32, 0x4E00..=0x9FFF)
}

use regex::Regex;
use std::collections::HashSet;

fn extract_hashtags(text: &str) -> HashSet<String> {
    let HASHTAG_REGEX: Regex = Regex::new(r"\#[a-zA-Z][0-9a-zA-Z_]*").unwrap();

    HASHTAG_REGEX
        .find_iter(text)
        .map(|mat| mat.as_str().to_string())
        .collect()
}

// Returns search on tags
// Removes tags from query
fn get_tag_filter(query: &mut String) -> Option<SearchRequest> {
    let tags = extract_hashtags(query);

    for tag in &tags {
        *query = query.replace(tag, "");
    }

    let queries: Vec<SearchRequest> = tags
        .iter()
        .map(|tag| {
            SearchRequest::Search(RequestSearchPart {
                terms: vec![tag.to_string()], // cut off hashtag
                path: "tags[]".to_owned(),
                ..Default::default()
            })
        })
        .collect();

    if !tags.is_empty() {
        Some(SearchRequest::Or(search::SearchTree {
            queries,
            options: Default::default(),
        }))
    } else {
        None
    }
}

use once_cell::sync::Lazy;
static PERSISTENCE: Lazy<Persistence> = Lazy::new(|| {
    persistence::Persistence::load(PathBuf::from("../create_db/indices/dict_velo")).unwrap()
});

static TO_ALTERNATIVE_VARIANT: Lazy<HashMap<char, char>> = Lazy::new(|| {
    vec![('気', '氣'), ('氷', '冰'), ('毎', '每')]
        .into_iter()
        .collect()
});

fn to_alternative_variant(kanji: char) -> char {
    TO_ALTERNATIVE_VARIANT.get(&kanji).copied().unwrap_or(kanji)
}

pub fn run_search_veloci(query: &str, top: usize) -> Result<SearchResultWithDoc, VelociError> {
    print_time!("SearchTime");

    let pers = &PERSISTENCE;

    let mut query = query.to_string();

    let tag_filter = get_tag_filter(&mut query);
    query = query.trim().to_string();
    dbg!(&query);

    let terms_from_query = || query.split_whitespace().filter(|el| !el.is_empty());

    let is_chinese_input = |term: &str| term.chars().any(is_chinese);

    let is_mixed_input = terms_from_query().any(is_chinese_input)
        && terms_from_query().any(|term| !is_chinese_input(term));

    //let num_terms = terms_from_query().count();
    let queries: Vec<SearchRequest> = terms_from_query()
        .flat_map(|term| {
            let fields = if is_chinese_input(term) {
                vec!["simplified", "traditional"]
            } else {
                vec![
                    "simplified",
                    "traditional",
                    //"pinyin",
                    "zhuyin",
                    //"pinyin_pretty",
                    "pinyin_search[]",
                    "tags[]",
                    "meanings[]",
                ]
            };

            let terms: Vec<(String, bool)> = if is_chinese_input(term) {
                // add regex
                // regular query just for boosting the exact match.
                // But not if mixed input, since in that case the user probably provides not exact
                // matches.
                if is_mixed_input {
                    vec![(format!(".*{}.*", term), true)]
                } else {
                    // single chinese character
                    if term.len() == 1 {
                        let orig_char = term.chars().next().unwrap();
                        let trad_char = to_alternative_variant(orig_char);
                        let mut chars = vec![orig_char, trad_char];
                        chars.dedup();
                        chars
                            .into_iter()
                            .flat_map(|cha| {
                                vec![(cha.to_string(), false), (format!(".*{}.*", cha), true)]
                            })
                            .collect()
                    } else {
                        // we replace the japanese chars with traditional ones. There's unlikely a
                        // match for japanese pairs
                        let term: String = term
                            .chars()
                            .map(|char| to_alternative_variant(char))
                            .collect();
                        vec![(term.to_string(), false), (format!(".*{}.*", term), true)]
                    }
                }
            } else {
                vec![(term.to_string(), false)]
            };

            fields.into_iter().flat_map(move |path| {
                terms.to_vec().into_iter().map(move |(term, is_regex)| {
                    SearchRequest::Search(RequestSearchPart {
                        terms: vec![term],
                        path: path.to_string(),
                        is_regex,
                        //levenshtein_distance: Some(0),
                        ..Default::default()
                    })
                })
            })
        })
        .collect();

    // Just search for the tags in case there's no search term and only tag filters
    let search_request: search::SearchRequest = if queries.is_empty() && tag_filter.is_some() {
        tag_filter.as_ref().cloned().unwrap()
    } else {
        SearchRequest::Or(search::SearchTree {
            queries,
            options: Default::default(),
        })
    };

    println!("{}", serde_json::to_string_pretty(&search_request).unwrap());

    let terms = terms_from_query().collect::<Vec<_>>();
    let phrase_queries = generate_phrase_queries_simple(
        pers,
        &terms,
        vec!["meanings[]".to_string(), "pinyin".to_string()],
    )
    .unwrap();
    println!(
        "phrase_queries {}",
        serde_json::to_string_pretty(&phrase_queries).unwrap()
    );

    let phrase_boosts = if phrase_queries.is_empty() {
        None
    } else {
        Some(phrase_queries)
    };

    let requesto = search::Request {
        why_found: true,
        filter: tag_filter.map(Box::new),
        search_req: Some(search_request),
        phrase_boosts,
        boost: Some(vec![
            RequestBoostPart {
                path: "commonness_boost".to_string(),
                boost_fun: Some(search::BoostFunction::Add),
                ..Default::default()
            },
            RequestBoostPart {
                path: "tocfl_level".to_string(), // levels 1-7. level1 is very common
                expression: Some("10 / $SCORE".to_string()),
                ..Default::default()
            },
        ]),
        top: Some(top),
        ..Default::default()
    };

    let res = search::to_search_result(
        pers,
        search::search(requesto.clone(), &pers).expect("search error"),
        &requesto.select,
    );
    //println!("{}", serde_json::to_string_pretty(&res).unwrap());
    //dbg!(&req);

    Ok(res)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mixed_search_hits() {
        let res = run_search_veloci("分 xiang3", 3).unwrap();
        assert_eq!(res.data[0].doc["traditional"], "分享");
    }

    #[test]
    fn test_fen() {
        let res = run_search_veloci("分", 3).unwrap();
        assert_eq!(res.data[0].doc["traditional"], "分");
        assert_eq!(res.data[0].doc["pinyin"], "fen1");
    }

    #[test]
    fn pinyin_search() {
        let pinyins = vec!["xiawu", "xia wu", "xiàwǔ", "xià wǔ", "xia4 wu3", "xia4wu3"];
        for pinyin in pinyins {
            let res = run_search_veloci(pinyin, 3).unwrap();
            assert_eq!(res.data[0].doc["traditional"], "下午");
        }
    }
}
