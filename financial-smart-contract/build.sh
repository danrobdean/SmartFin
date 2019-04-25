#!/bin/bash

cd contract

# Generate new build cargo manifest (just in case)
source ./generate-cargo.sh cdylib

# Build
cargo build --release --target wasm32-unknown-unknown
wasm-build --target=wasm32-unknown-unknown ./target financial_smart_contract

cd ..

cp ./contract/target/json/FinancialScInterface.json ./contract-js-test/resources/abi.json
cp ./contract/target/financial_smart_contract.wasm ./contract-js-test/resources/contract.wasm