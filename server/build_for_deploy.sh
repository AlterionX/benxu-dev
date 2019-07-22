#!/usr/bin/env sh

rm -rf public/wasm
rm -rf public/js/wasm-bindgen-glue
cargo build --release --out-dir=. -Z unstable-options
