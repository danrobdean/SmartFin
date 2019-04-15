#!/bin/bash

# Generate new build cargo manifest (just in case)
source ./generate-cargo.sh cdylib

# Build
cargo build --release --target wasm32-unknown-unknown
wasm-build --target=wasm32-unknown-unknown ./target financial_smart_contract