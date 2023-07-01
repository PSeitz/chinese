#!/bin/bash

cd create_json/;cargo run --release; cd ..;create_index --data ./create_json/db.json --target indices/dict_velo --config indices/veloci_config.toml
