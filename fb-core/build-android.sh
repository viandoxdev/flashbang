#!/bin/bash

# requires cargo-ndk and the following rustup targets:
# - aarch64-linux-android 
# - armv7-linux-androideabi 
# - i686-linux-android 
# - x86_64-linux-android

# Build the dylib
cargo build
# Build library
cargo ndk -o ../app/src/main/jniLibs \
    --manifest-path ./Cargo.toml \
    -t armeabi-v7a \
    -t arm64-v8a \
    -t x86 \
    -t x86_64 \
    build --release
# Generate bindgens
cargo run --bin uniffi-bindgen generate \
    --library ./target/debug/libmobile.so \
    --language kotlin \
    --out-dir ../app/src/main/java/dev/vndx/flashbang/rust
