#!/bin/bash

cd $PWD

cd contract

# Generate new build cargo manifest (just in case)
source ./generate-cargo.sh cdylib

# Build
cargo build --release --target wasm32-unknown-unknown
wasm-build --target=wasm32-unknown-unknown ./target financial_smart_contract

cd -

mkdir -p ./client/resources
cp ./contract/target/json/FinancialScInterface.json ./client/resources/abi.json
cp ./contract/target/financial_smart_contract.wasm ./client/resources/contract.wasm

cd client

# Export resources as javascript variables
./generate-resources.sh

cd -

cd -