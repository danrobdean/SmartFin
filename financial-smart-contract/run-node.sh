#!/bin/bash

# Runs the parity node
start-node () {
    parity --chain ./wasm-dev-chain.json --jsonrpc-apis=all --jsonrpc-cors=all --geth
}

# Cleans the blockchain
clean-chain () {
    rm -rf ~/.local/share/io.parity.ethereum/*
}

# Initialises an admin account on the blockchain after a delay
init-account () {
    sleep 2
    curl --data '{"jsonrpc":"2.0","method":"parity_newAccountFromPhrase","params":["user", "user"],"id":0}' -H "Content-Type: application/json" -X POST localhost:8545
    echo "Account initialised."
}

# Initialises two test accounts on the blockchain after a delay
init-test-accounts() {
    sleep 2
    curl --data '{"jsonrpc":"2.0","method":"parity_newAccountFromSecret","params":["0xe919476cbaf686942002394228dc7a2c1f41ed01ef80744e24d156ac00df1fc8", "test"],"id":1}' -H "Content-Type: application/json" -X POST localhost:8545
    echo "Account 1 initialised."
    curl --data '{"jsonrpc":"2.0","method":"parity_newAccountFromSecret","params":["0xfa1bf117f434f89e7be33bc480b0744575f58acd2c9e1da728587be8265dcd05", "test"],"id":2}' -H "Content-Type: application/json" -X POST localhost:8545
    echo "Account 2 initialised."
    curl --data '{"jsonrpc":"2.0","method":"parity_newAccountFromSecret","params":["0x688df665ef8e734c4dd86f7c759a0a50f7dadfe2638b550ab01b44ff4dbf57c5", "test"],"id":3}' -H "Content-Type: application/json" -X POST localhost:8545
    echo "Account 3 initialised."
}

cd "$(dirname "$0")"

# Print help if requested
if [ "$1" = "-h" ]; then
    echo "To run a clean node, pass --clean."
else
    # Clean the blockchain if requested
    if [ "$1" = "--clean" ]; then
        clean-chain
        echo "Chain cleaned."
    fi
    # Initialise the account after a delay, and start the parity node
    init-account & init-test-accounts & start-node
fi

cd -
