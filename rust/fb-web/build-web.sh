#!/bin/bash

wasm-pack build --target bundler --profile release-web --out-dir pkg
zip -r wasm-bundle.zip pkg/
