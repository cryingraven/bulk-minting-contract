#!/bin/bash
set -e
cd "`dirname $0`"
#export RUSTFLAGS='-C link-arg=-s'
cargo build  --release --all --target wasm32-unknown-unknown
cp target/wasm32-unknown-unknown/release/*.wasm ./out/main.wasm