#!/usr/bin/env sh

wasm-pack build --target no-modules --out-dir ../server/public/wasm-pack "$@"

