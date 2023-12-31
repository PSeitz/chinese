
# Chisho
Chisho is a Chinese dictionary

http://chisho.org/

### Discord
https://discord.gg/JmP8gW6EdB

# Setup

```bash
git clone https://github.com/cjhoward/cedict-tts.git
git clone https://github.com/gnuish/pinyin-zhuyin.git

cargo install --git https://github.com/PSeitz/veloci.git veloci_bins --bin create_index
cd create_db/create_json/;cargo run --release; cd ..;create_index --data ./create_json/db.json --target indices/dict_velo --config indices/veloci_config.toml
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

