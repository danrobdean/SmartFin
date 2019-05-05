#!/bin/bash

# Get resources
ABI="$(cat ./resources/abi.json)"
CONTRACT="$(xxd -p -c 100000000 ./resources/contract.wasm)"

# Save to mjs file
cat > ./resources/resources.mjs <<-EOF
export const ABI = $ABI;

export const CODE_HEX = '0x$CONTRACT';
EOF