

# Setup

```
git clone https://github.com/cjhoward/cedict-tts.git
git clone https://github.com/gnuish/pinyin-zhuyin.git
cd create_db
git clone https://github.com/PSeitz/tocfl.git
```

# Contents

## create_db

- Creates a JSON database of the CC-CEDICT dictionary.
- Creates a search index with veloci based on the JSON database.

### create_db/ch_freq

Build a frequency list of Chinese characters. This is used to boost entries in the dictionary.

### create_db/create_json
- Creates a JSON database of the CC-CEDICT dictionary, but annotated with additional information like pinyin, zhuyin, and frequency.


## webpage

The webpage to search the dictionary created in create_db.

See [webpage/TODO.md](webpage/TODO.md) for more implemented and planned features.

