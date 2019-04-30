#!/bin/bash

test-rs() {
    cd contract
    ./test.sh
    cd ..
}

test-js() {
    ./build.sh
    cd contract-js-test
    yarn test
    cd ..
}

if [ "$1" = "-rs" ]
then
    test-rs
elif [ "$1" = "-js" ]
then
    test-js
else
    test-rs
    test-js
fi