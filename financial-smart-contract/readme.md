## Financial Smart Contract
This directory contains the financial smart contract, written in Rust, and several development tools.

### Building the contract

To build the smart contract, run `build.sh` from this directory. This will output the `target` directory.

### Testing the contract

To test the contract, run `test.sh` from this directory. This will run any unit tests (defined with `pwasm-test`).

### Running the development blockchain

To run the development blockchain (defined in `wasm-dev-chain.json`), execute `run-node.sh` from this directory. The blockchain can be cleaned and then run with `run-node.sh --clean`.

Once the blockchain is running, the contract can be deployed from a node console.

First, execute `yarn install` to install web3 dependencies.

Next, run `node`, and then execute `.load deploy-contract.js` to load a javascript file which defines several functions/imports for interacting with the contract.

Call `unlockAccount()` to unlock a test account with some ether, and then call `loadAndDeployContract()` to deploy the contract from the default output location (this may fail due to gas estimation, in which case call the function again).

