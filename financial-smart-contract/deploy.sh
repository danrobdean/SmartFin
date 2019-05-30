#!/bin/bash

cd "$(dirname "$0")"/client

node --experimental-modules ./src/js/deploy-contract.mjs

cd -