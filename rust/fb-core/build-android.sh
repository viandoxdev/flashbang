#!/bin/bash

# requires cargo-ndk and the following rustup targets:
# - aarch64-linux-android 
# - armv7-linux-androideabi 
# - i686-linux-android 
# - x86_64-linux-android

# I'm keeping one of these because I don't use them and I need speed

# Build the dylib
cargo build
# Build library
cargo ndk -o ../app/src/main/jniLibs \
    --manifest-path ./Cargo.toml \
    -t arm64-v8a \
    build --release
# Generate bindgens
mkdir -p ../app/src/main/java/dev/vndx/flashbang/rust
cargo run --bin uniffi-bindgen generate \
    --library ./target/debug/libmobile.so \
    --language kotlin \
    --out-dir ../app/src/main/java/dev/vndx/flashbang/rust
