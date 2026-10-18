#!/bin/sh
set -e
cd /home/zaratan/Projects/dvr-wt/lancement/out/347/ws
V=/home/zaratan/Projects/dvr-wt/verrou
CARGO_TARGET_DIR=../target $V lourd cargo build --release --target wasm32-unknown-unknown -p webproto --lib
RUSTFLAGS="-C target-feature=+simd128" CARGO_TARGET_DIR=../target-simd $V lourd cargo build --release --target wasm32-unknown-unknown -p webproto --lib
RUSTFLAGS="-C target-feature=+atomics,+bulk-memory,+simd128 -C link-arg=--shared-memory -C link-arg=--max-memory=4294967296 -C link-arg=--import-memory -C link-arg=--export=__wasm_init_tls -C link-arg=--export=__tls_size -C link-arg=--export=__tls_align -C link-arg=--export=__tls_base" CARGO_TARGET_DIR=../target-threads $V lourd cargo +nightly build --release --target wasm32-unknown-unknown -p webproto --lib --features threads -Z build-std=panic_abort,std
cd ..
tools/bin/wasm-bindgen --target web --out-dir pkg-single target/wasm32-unknown-unknown/release/webproto.wasm
tools/bin/wasm-bindgen --target web --out-dir pkg-simd target-simd/wasm32-unknown-unknown/release/webproto.wasm
tools/bin/wasm-bindgen --target web --out-dir pkg-threads target-threads/wasm32-unknown-unknown/release/webproto.wasm
