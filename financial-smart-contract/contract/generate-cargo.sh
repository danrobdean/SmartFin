#!/bin/bash

# parameter for crate-type ("cdylib" for build, "lib" for testing)
CRATE_TYPE=$1

# Remove existing Cargo.toml
rm -f ./Cargo.toml

# Output filled-in Cargo.toml template (fill in crate-type, remove comment lines)
sed -e "s/\${crate-type}/$CRATE_TYPE/" -e"s/#.*$//" cargo-template.toml > ./Cargo.toml