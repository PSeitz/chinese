#![allow(non_snake_case)]
use core::fmt;
use std::path::PathBuf;
use std::str::FromStr;

use axum::{extract::Query, response::Html, routing::get, Router};
use dioxus::prelude::*;
//use search::run_search;
use prettify_pinyin::prettify;
use serde::{de, Deserialize, Deserializer, Serialize};
//use tantivy::{
//schema::{NamedFieldDocument, Schema},
//Document,
//};
use measure_time::*;
use tower_http::services::{ServeDir, ServeFile};

mod search;

//use dioxus_router::{Route, Router};
//use axum_server::tls_rustls::RustlsConfig;
use urlencoding::encode;

use crate::search::run_search_veloci;
const APP_NAME: &str = "Chisho";

//use syslog::{BasicLogger, Facility, Formatter3164};

#[tokio::main]
async fn main() {
    env_logger::init();
    //let formatter = Formatter3164 {
    //facility: Facility::LOG_USER,
    //hostname: None,
    //process: "chinese_web".into(),
    //pid: 0,
    //};

    //let logger = syslog::unix(formatter).expect("could not connect to syslog");
    //log::set_boxed_logger(Box::new(BasicLogger::new(logger)))
    //.map(|()| log::set_max_level(LevelFilter::Info))
    //.unwrap();

    let env_port = std::env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    // parsed
    let port = env_port.parse::<u16>().unwrap_or(3000);
    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], port));
    //let addr = std::net::SocketAddr::from(([127, 0, 0, 1], 3000));
    info!("listening on http://{}", addr);

    // Check if env var CERT_PATH is set
    //let config = if let Ok(cert_path) = std::env::var("CERT_PATH") {
    //let config = RustlsConfig::from_pem_file(
    //PathBuf::from(cert_path.to_owned()).join("fullchain.pem"),
    //PathBuf::from(cert_path).join("privkey.pem"),
    //)
    //.await
    //.unwrap();
    //config
    //} else {
    //let subject_alt_names = vec!["chisho.org".to_string(), "localhost".to_string()];
    //let cert = rcgen::generate_simple_self_signed(subject_alt_names).unwrap();
    //let config = RustlsConfig::from_pem(
    //cert.serialize_pem().unwrap().as_bytes().to_vec(),
    //cert.serialize_private_key_pem().as_bytes().to_vec(),
    //)
    //.await
    //.unwrap();
    //config
    //};

    let serve_dir = ServeDir::new("dist").not_found_service(ServeFile::new("dist/output.css"));
    let media_dir =
        ServeDir::new("../cedict-tts/").not_found_service(ServeFile::new("dist/output.css"));
    //axum_server::bind_rustls(addr, config)
    axum::Server::bind(&addr)
        .serve(
            Router::new()
                //.route("/", get(app_ssr))
                .route("/", get(app_endpoint))
                .route("/about", get(app_endpoint))
                .nest_service("/dist", serve_dir.clone())
                .nest_service("/media", media_dir.clone())
                .into_make_service(),
        )
        .await
        .unwrap();

    //dioxus_web::launch(app_ssr);
}

//pub fn app(cx: Scope) -> Element {
//cx.render(rsx! {
//link {
//rel: "stylesheet",
//href: "/dist/output.css",
//},
//Router {
//div {
//class: "flex items-center max-w-md mx-auto flex-col",

//SearchResult { entries: vec![] }
//Route { to: "/", About {} }
//Route { to: "/index.html", About {} }
//}
//}
//Footer {}
//})
//}

//object.onclick = function(){myScript};

fn render_page(search_term: String, ssr_output: String) -> Html<String> {
    let title = if search_term.is_empty() {
        "Chisho.org - Chinese Dictionary".to_string()
    } else {
        format!("{} - Chisho.org", search_term)
    };
    let audio_script = r#"
   <script>

        function ready(fn) {
          if (document.readyState !== 'loading') {
            fn();
          } else {
            document.addEventListener('DOMContentLoaded', fn);
          }
        }
        ready(attachButtons)
        ready(set_search_input)
        function attachButtons(){
            let allbtns = document.querySelectorAll('button');
            for (let btn of allbtns) {
                for (let clazz of btn.classList) {
                    if (clazz.startsWith("attach_to_")) {
                        let id = clazz.substring("attach_to_".length)
                        btn.onclick = function(){document.getElementById(id).play()};
                    }
                }
            }
        }
        function set_search_input() {
            let query = new URLSearchParams(window.location.search).get("q");
            document.querySelector("\#search_input").value = query;
            
        }
    </script>


        "#;
    Html(format!(
        r#"
<!DOCTYPE html>
<html data-theme="emerald" lang="en">
  <head>
    <link rel="icon" href="dist/favicon.svg"/>
    <link rel="stylesheet" href="/dist/output.css">
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <meta http-equiv="X-UA-Compatible" content="ie=edge">
{audio_script}
    <title>{title}</title>
    <style>
        fg:before {{
          content: attr(t);
          display: block;
            font-size: 50%;
            text-align: start;
          line-height: 1.5;
        }}

        fg {{
          display: inline-block;
          text-indent: 0px;
          line-height: normal;
            -webkit-text-emphasis: none;
          text-align: center;
          line-height: 1;
        }}
    </style>
  </head>
  <body>
    {ssr_output}
  </body>
</html>
"#
    ))
}

#[derive(Debug, PartialEq, Props, Deserialize)]
#[allow(dead_code)]
struct Params {
    #[serde(default, deserialize_with = "empty_string_as_none")]
    q: Option<String>,

    #[serde(default, deserialize_with = "empty_string_as_none")]
    top: Option<usize>,
}

/// Serde deserialization decorator to map empty Strings to None,
fn empty_string_as_none<'de, D, T>(de: D) -> Result<Option<T>, D::Error>
where
    D: Deserializer<'de>,
    T: FromStr,
    T::Err: fmt::Display,
{
    let opt = Option::<String>::deserialize(de)?;
    match opt.as_deref() {
        None | Some("") => Ok(None),
        Some(s) => FromStr::from_str(s).map_err(de::Error::custom).map(Some),
    }
}

async fn app_endpoint(params: Query<Params>) -> Html<String> {
    let search_term = params
        .q
        .as_ref()
        .map(ToString::to_string)
        .unwrap_or_default();
    debug_time!("Render Page Time");
    render_page(
        search_term.to_string(),
        dioxus_ssr::render_lazy(rsx! {
            Page{q: search_term, top: params.top.unwrap_or(20)}
            //Page{q: params.q.as_ref().unwrap_or(&"".to_string()).to_string(), top: }
        }),
    )
}

const LINK_CLASSES: &str = "underline text-slate-500 hover:text-blue-600 ";

#[derive(Clone, Debug, PartialEq, Props, Deserialize, Serialize)]
pub struct Entry {
    simplified: String,
    traditional: String,
    simplified_radicals: Option<Vec<Vec<String>>>,
    traditional_radicals: Option<Vec<Vec<String>>>,
    pinyin: String,
    pinyin_taiwan: Option<String>,
    // different pinyin variants for search. this could be covered by
    // tokenization but that's simpler
    pinyin_search: Vec<String>,
    zhuyin: String,
    pinyin_pretty: String,
    tocfl_level: Option<u32>,
    meanings: Vec<String>,
    meanings_de: Vec<String>,
    tags: Vec<String>,
    commonness_boost: f64,
    count_per_million_written: u64,
    count_per_million_spoken: u64,
    count_per_million_in_others: u64,
}

fn Page(cx: Scope<Params>) -> Element {
    let term = cx.props.q.to_owned().unwrap_or("".to_string());
    let top = cx.props.top.to_owned().unwrap_or(20);
    let req = if !term.is_empty() {
        let res = run_search_veloci(&term, top);
        match res {
            Ok(res) => res,
            Err(e) => {
                error!("Error in search: {:?}", e);
                //return cx.render(rsx!(div { "Error in search {}", e }));
                return cx.render(rsx!(
                    div{
                        class:"container mx-auto px-4 max-w-screen-md",
                        Logo{}
                        SearchInput{input_value: term.to_string()}
                        cx.render(rsx!(div { "{e}" }))
                    }
                ));
            }
        }
    } else {
        Default::default()
    };

    let entries = req
        .data
        .iter()
        .map(|hit| serde_json::from_str(&serde_json::to_string(&hit.doc).unwrap()).unwrap())
        .collect::<Vec<_>>();
    let has_query = !term.is_empty();

    cx.render(rsx!(
        div{
            class:"container mx-auto px-4 max-w-screen-md",
            Logo{}
            SearchInput{input_value: term.to_string()}
            if has_query{
                cx.render(rsx! {
                    SearchResult {entries: entries, num_results: req.num_hits, current_query: term, top: top},
                })
            }
            if !has_query{
                cx.render(rsx! {
                    StartPage {}
                })
            }
        }
    ))
}

pub fn Logo(cx: Scope) -> Element {
    cx.render(rsx!(

        a{
            class: "logo-container text-2xl font-serif font-bold logo-text",
            href: "/",
            "Chisho",

            svg {
                id: "Layer_1",
                class: "logo-svg-container",
                height: "25",
                preserve_aspect_ratio: "xMinYMin",
                style: "overflow: visible;",
                version: "1.1",
                view_box: "0 0 100 90",
                x: "162.2",
                "xml:space": "preserve",
                xmlns: "http://www.w3.org/2000/svg",
                "xmlns:xlink": "http://www.w3.org/1999/xlink",
                path {
                    class: "st4",
                    d: "M76.2,30.7V7.1H14.5c-5.2,0-9.4,4.3-9.4,9.4v4.7c0,5.2,4.3,9.4,9.4,9.4h11.1c-5.2,0-9.4,4.3-9.4,9.4v4.7   c0,5.2,4.3,9.4,9.4,9.4H14.5c-5.2,0-9.4,4.3-9.4,9.4v4.7c0,5.2,4.3,9.4,9.4,9.4h61.7V54.3h11.1V30.7H76.2z",
                    style: "fill: #FFFFFF;",
                }
                path {
                    class: "st46",
                    d: "M42,42.5v21.7c0,0.9,0.5,1.8,1.3,2.2c0.8,0.4,1.8,0.4,2.6-0.1l5.9-3.9l5.9,3.9c0.4,0.3,0.9,0.4,1.4,0.4   c0.4,0,0.8-0.1,1.2-0.3c0.8-0.4,1.3-1.3,1.3-2.2V42.5H42z",
                    style: "fill: #E15E31;",
                }
                path {
                    class: "st46",
                    d: "M72.2,74H14.5c-3,0-5.4-2.4-5.4-5.4v-4.7c0-3,2.4-5.4,5.4-5.4H37v-8H14.5c-7.4,0-13.4,6-13.4,13.4v4.7   c0,7.4,6,13.4,13.4,13.4h61.7c2.2,0,4-1.8,4-4V66.3h-8V74z",
                    style: "fill: #376596; fill-opacity: 1;",
                }
                path {
                    class: "st145",
                    d: "M83.3,50.3H66.5v8h20.8c2.2,0,4-1.8,4-4V42.7h-8V50.3z M91.3,26.7H25.6c-7.4,0-13.4,6-13.4,13.4v4.7   c0,7.4,6,13.4,13.4,13.4H37v-8H25.6c-3,0-5.4-2.4-5.4-5.4v-4.7c0-3,2.4-5.4,5.4-5.4h65.7c2.2,0,4-1.8,4-4S93.5,26.7,91.3,26.7z",
                    style: "fill: #49a0d9; fill-opacity: 1;",
                }
                path {
                    class: "st301",
                    d: "M14.5,11.1h65.7c2.2,0,4-1.8,4-4s-1.8-4-4-4H14.5c-7.4,0-13.4,6-13.4,13.4v4.7c0,7.4,6,13.4,13.4,13.4h61.7   c2.2,0,4-1.8,4-4V19.1h-8v7.6H14.5c-3,0-5.4-2.4-5.4-5.4v-4.7C9.1,13.5,11.5,11.1,14.5,11.1z",
                    style: "fill: #6FBDD3;",
                }
            }
        }

    ))

    //cx.render(rsx!(
    //div {
    //class: "grow",
    //h1 {
    //class: "place-self-start",
    //a {
    //class: "text-2xl font-serif font-bold",
    //href: "/","Chisho"
    //}
    //span {
    //class: "text-xs",
    //"Dictionarysss",
    //}
    //}
    //}
    //))
}

// Remember: Owned props must implement `PartialEq`!
#[derive(PartialEq, Props)]
pub struct InputParams {
    input_value: String,
}

pub fn SearchInput(cx: Scope<InputParams>) -> Element {
    cx.render(rsx!(
        div {
            class: "grow",
            form {
                div {
                    class: "flex mx-auto",
                    input {
                        id: "search_input",
                        class: "bg-transparent text-gray-700 w-full focus:outline-none focus:shadow-outline border border-gray-300 rounded-lg py-2 px-4 block appearance-none leading-normal",
                        value: "{cx.props.input_value}",
                        name: "q",
                        placeholder: "Chinese, English, pinyin, zhuyin",
                        r#type: "text",
                        autofocus: true
                    }
                    button {
                        class: "btn",
                        "Search"
                    }
                }
            }
        }
    ))
}

// Remember: Owned props must implement `PartialEq`!
#[derive(PartialEq, Props)]
pub struct SearchResultProps {
    entries: Vec<Entry>,
    num_results: u64,
    current_query: String,
    top: usize,
}

pub fn SearchResult(cx: Scope<SearchResultProps>) -> Element {
    let new_top = cx.props.top + 20;
    let q = &cx.props.current_query;
    cx.render(rsx!(
        div {
            class: "grow",
            div { class:"text-sm text-slate-400", "{cx.props.num_results} Results" }

            div {
                class: "p-1",
                ul {
                    for entry in &cx.props.entries {
                        li { SearchResultItem{entry:entry.clone(), current_query: cx.props.current_query.to_string()} }
                    }
                }
            }
            if (cx.props.entries.len() as u64) < cx.props.num_results {
                cx.render(rsx! {
                   a{ href:"{get_search_url_with_top(q, \"\", new_top)}",  "More Words >"}
                })
            }
        }
    ))
}

// Remember: Owned props must implement `PartialEq`!
#[derive(PartialEq, Props)]
pub struct SearchResultItemProp {
    entry: Entry,
    current_query: String,
}

pub fn SearchResultItem(cx: Scope<SearchResultItemProp>) -> Element {
    let entry = &cx.props.entry;
    let q = &cx.props.current_query;

    //let audio_path = format!("../../")

    let simpl_part = if entry.traditional == entry.simplified {
        "".to_string()
    } else {
        let same_prefix = entry
            .traditional
            .chars()
            .zip(entry.simplified.chars())
            .take_while(|(el1, el2)| el1 == el2)
            .count();

        let same_suffix = entry
            .traditional
            .chars()
            .rev()
            .zip(entry.simplified.chars().rev())
            .take_while(|(el1, el2)| el1 == el2)
            .count();

        if same_prefix != 0 {
            let diff_prefix = entry
                .simplified
                .chars()
                .skip(same_prefix)
                .collect::<String>();
            format!("〔-{}〕", diff_prefix)
        } else if same_suffix != 0 {
            let diff_suffix = entry
                .simplified
                .chars()
                .take(entry.simplified.chars().count() - same_suffix)
                .collect::<String>();
            format!("〔{}-〕", diff_suffix)
        } else {
            format!("〔{}〕", entry.simplified)
        }
    };

    let pinyin_no_ws: String = entry
        .pinyin
        .chars()
        .filter(|c| !c.is_whitespace())
        .collect::<String>()
        .to_lowercase();

    let has_audio_male = std::path::Path::new(&format!("../cedict-tts/male/{}.mp3", pinyin_no_ws))
        .try_exists()
        .unwrap();
    let has_audio_female =
        std::path::Path::new(&format!("../cedict-tts/female/{}.mp3", pinyin_no_ws))
            .try_exists()
            .unwrap();

    let mut audios = Vec::new();
    if has_audio_female {
        audios.push((
            uuid::Uuid::new_v4(),
            format!("media/female/{}.mp3", pinyin_no_ws),
        ));
    }

    if has_audio_male {
        audios.push((
            uuid::Uuid::new_v4(),
            format!("media/male/{}.mp3", pinyin_no_ws),
        ));
    }

    //<fg t="わたし">私</fg>

    let mut pinyin = entry.pinyin_pretty.to_string();
    let zhuyin = entry.zhuyin.to_string();
    if let Some(pinyin_taiwan) = entry.pinyin_taiwan.as_ref() {
        pinyin += &(" / ".to_string() + &prettify(pinyin_taiwan.to_string()));
    }
    //if pinyin != zhuyin {
    //pinyin += &(" / ".to_string() + &zhuyin.to_string());
    //}
    cx.render(rsx!(
        div { class:"flex flex-row mt-2",
            div { class:"basis-1/4 pl-1",

            div{
                //ruby{ class:"text-3xl font-medium", "{entry.traditional}" rt{ "{pinyin}" } } span{ class:"text-3xl font-medium", "{simpl_part}"}

                span{ class:"chinese-word",
                span { class: "annotation text-sm font-medium", "{pinyin}" }
                //span { class: "annotation text-sm font-medium", "{zhuyin}" }
                    span { class: "character text-3xl font-medium", "{entry.traditional} {simpl_part}" }
                }


                for audio in audios.iter() {
                    cx.render(rsx! {
                        audio {
                            id: "{audio.0}",
                            src: "{audio.1}",
                        }
                        p{
                            button {
                                class: "attach_to_{audio.0} {LINK_CLASSES} text-sm",
                                "Play Audio"
                            }
                        }
                    })
                }

            }
            progress {
                class: "progress w-56",
                max: "10", // Actual max is higher
                value: "{entry.commonness_boost - 1.0}",
            }
            div{
                for tag in entry.tags.iter().filter(|tag|tag.as_str() != "TOCFL") {
                    cx.render(rsx! {
                        //a { href:"/?q={encode(q)}+{encode(tag)}", class:"badge badge-primary mr-1", "{tag}"}
                        //a { href:"{get_search_url(q, tag)}", class:"badge badge-primary mr-1", style:"background-color:{generate_color_hash(tag)};border-color:{generate_color_hash(tag)};", "{tag}" }
                        a { href:"{get_search_url(q, tag)}", class:"badge badge-primary mr-1", "{tag}" }
                    })
                }
            }
            },
            div { class:"basis-3/4 pl-1",
                for (i, def) in entry.meanings.iter().enumerate() {
                    div { "{i+1}. {def}" }
                }
                    br { }
                //for (i, def) in entry.meanings_de.iter().enumerate() {
                    //div { "{i+1}. {def}" }
                //}
            },
        }
        div { class: "divider" }
    ))
}

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

fn generate_color_hash(input: &String) -> String {
    let mut hasher = DefaultHasher::new();
    input.hash(&mut hasher);
    let hash = hasher.finish();

    format!("#{:06x}", hash & 0xFFFFFF)
}

fn get_search_url(q: &str, tag: &str) -> String {
    format!("/?q={}+{}", encode(q), encode(tag))
}

fn get_search_url_with_top(q: &str, tag: &str, top: usize) -> String {
    format!("/?q={}+{}&top={}", encode(q), encode(tag), top)
}

struct Example {
    desc: String,
    examples: Vec<(String, String)>, //url: string,
                                     //url_name: String,
}
impl Example {
    fn new(desc: &str, url: &str, url_name: &str) -> Self {
        Self {
            desc: desc.to_string(),
            examples: vec![(url.to_string(), url_name.to_string())],
        }
    }

    fn new_multi(desc: &str, examples: Vec<(String, String)>) -> Self {
        Self {
            desc: desc.to_string(),
            examples,
        }
    }
}
pub fn StartPage(cx: Scope) -> Element {
    let examples = vec![
        Example::new(
            "Great English search: ",
            &get_search_url("home", ""),
            "home",
        ),
        Example::new("TOCFL words: ", &get_search_url("", "#TOCFL1"), "#TOCFL1"),
        Example::new(
            "Filter for common words: ",
            &get_search_url("", "#common"),
            "#common",
        ),
        Example::new(
            "Mix chinese and pinyin: ",
            &get_search_url("分 xiang3", ""),
            "分 xiang3",
        ),
        Example::new_multi(
            "Different variants of pinyin: ",
            vec![
                (get_search_url("xiawu", ""), "xiawu".to_string()),
                (get_search_url("xia wu", ""), "xia wu".to_string()),
                (get_search_url("xià wǔ", ""), "xià wǔ".to_string()),
                (get_search_url("xia4 wu3", ""), "xia4 wu3".to_string()),
            ],
        ),
        Example::new_multi(
            "Search with zhuyin: ",
            vec![(get_search_url("ㄒㄧㄚˋ ㄨˇ", ""), "ㄒㄧㄚˋ ㄨˇ".to_string())],
        ),
    ];

    cx.render(rsx!(div{
        class: "m-4 flex justify-center leading-loose",
        div{
            class: "max-w-lg",
            p {
                class: "",
                "{APP_NAME} is a powerful Chinese-English dictionary. It lets you find words, chinese characters, pinyin, zhuyin quickly and easily. It's like pleco but for the web, or Jisho for Chinese."
            }
            p {
                class: "mt-4",
                "Here are some examples on what {APP_NAME} can do"
            }

            ul{ class: "mt-2 list-disc list-inside",
                for example in examples.iter() {
                    cx.render(rsx! {
                        li{  "{example.desc} ", 

                            for el in example.examples.iter() {
                                cx.render(rsx! {
                                    a { class:"{LINK_CLASSES} pr-1 ", href:"{el.0}", "{el.1}"}
                                })
                            }

                        }

                    })
                }

               //p{ "Usage Tips: Add a tag to list only common entries: e.g. \"home #common\""}
               //p{ "List of Tags: [p{#common}, #TOCL, #TOCL1, #TOCL2, #TOCL3, #TOCL4, #TOCL5, #TOCL6]" }
            }
        }
    }))
}

pub fn Footer(cx: Scope) -> Element {
    cx.render(rsx!(p {}))
}
