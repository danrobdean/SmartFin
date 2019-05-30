#!/bin/bash
cd "$(dirname "$0")"

cd client
yarn build
cd -

rm -f ./dist-client/resources/*
cp ./client/dist/* ./dist-client/resources

cd -