#!/bin/bash

test-rs() {
    cd contract
    "./test.sh"
    cd -
}

test-js() {
    "./build.sh"
    cd client
    yarn test
    cd -
}

cd "$(dirname "$0")"

RS_TEST=0
JS_TEST=0
NC_TEST=0

while test $# -gt 0
do
    case "$1" in
        -rs) RS_TEST=1
            ;;
        -js) JS_TEST=1
            ;;
        -nc) NC_TEST=1
            ;;
    esac
    shift
done

if [ $RS_TEST -eq 1 ]
then
    test-rs
else
    # Start blockchain node if required
    if [ $NC_TEST -eq 0 ]
    then
        "./run-node.sh" &
        sleep 3
    fi

    # Run tests
    if [ $JS_TEST -eq 1 ]
    then
        test-js
    else
        test-rs
        test-js
    fi

    # Kill blockchain node if required
    if [ $NC_TEST -eq 0 ]
    then
        kill -9 $(lsof -ti :8545)
    fi
fi

cd -