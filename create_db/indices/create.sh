#!/bin/bash

rm -rf dict;mkdir dict;cp dict.meta dict/meta.json;cat ../create_json/db.json | tantivy index -m 4000000000 -t 1 -i dict;du -s dict/*
