#!/bin/bash
cd "$(dirname "$0")"

./build.sh

cd client
yarn install
yarn build
cd -

rm -f ./dist-client/resources/*
cp ./client/dist/* ./dist-client/resources

cd -