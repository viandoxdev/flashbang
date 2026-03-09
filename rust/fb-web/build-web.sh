#!/bin/bash

wasm-pack build --target bundler --release --out-dir pkg
zip -r wasm-bundle.zip pkg/
