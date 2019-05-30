#!/bin/bash

run-client() {
    cd client

    yarn start

    cd -
}

cd "$(dirname "$0")"

# -nc (no-chain), run without starting a new blockchain node
if [ "$1" = "-nc" ]
then
    run-client
else
    "./run-node.sh" "--clean" &
    sleep 3
    run-client
    kill -9 $(lsof -ti :8545)
fi

cd -