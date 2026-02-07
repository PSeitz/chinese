
# Data Sources For `db.json`

## `../cedict_ts.u8`
Origin: CC-CEDICT (Chinese-English), https://www.mdbg.net/chinese/dictionary?page=cedict
Fields: `simplified`, `traditional`, `pinyin`, `meanings`, `pinyin_ws_tone_number`
Derived in code from CEDICT fields: `pinyin_taiwan` (from Taiwan pr. in definitions or inferred by single-character entries)

## `../handedict.u8`
Origin: HanDeDict (Chinese-German), https://handedict.zydeo.net/de/download
Fields: `meanings_de`

## `kanji.json`
Origin: `davidluzgouveia/kanji-data`
Fields: `kanji` (strokes/grade/frequency/readings, WaniKani metadata, etc.)
Derived from kanji data: `tags` (`#WK`, `#WaniKaniLevel{N}`)

## `traditional_character_radicals.txt`
Origin: https://github.com/kfcd/chaizi
Fields: `traditional_radicals`

## `simplified_character_radicals.txt`
Origin: https://github.com/kfcd/chaizi
Fields: `simplified_radicals`

## `tocfl` crate
Origin: TOCFL frequency/levels compiled from official benchmark lists
See: `../tocfl/Vocabulary_List_111-11-14.xlsx`, `../tocfl/Chinese_Character_List_111-09-20.xlsx`
Fields: `tocfl_level`, `count_per_million_written`, `count_per_million_spoken`, `count_per_million_in_others`
Derived from TOCFL: `commonness_boost`, commonness tags (`#common`, `#common_written`, `#common_spoken`, `#verycommon`, `#commonchar`), TOCFL tags (`#TOCFL`, `#TOCFL{N}`)

## Derived in code (no external file)
Source: `prettify_pinyin` crate
Fields: `pinyin_pretty`

Source: `pinyin_zhuyin` crate
Fields: `zhuyin`

Source: internal transforms
Fields: `pinyin_search` (variants from `pinyin_ws_tone_number` and `pinyin_taiwan`)
