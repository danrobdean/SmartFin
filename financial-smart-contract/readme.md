## Financial Smart Contract
This directory contains the financial smart contract, written in Rust, and several development tools.

### Building the smart contract

To build the smart contract, run `build.sh` from this directory. This will output the `target` directory.

### Testing the smart contract

To test the smart contract, run `test.sh` from this directory. This will run any unit tests (defined with `pwasm-test`).

### Running the development blockchain

To run the development blockchain (defined in `wasm-dev-chain.json`), execute `run-node.sh` from this directory. The blockchain can be cleaned and then run with `run-node.sh --clean`.

Once the blockchain is running, the contract can be deployed from a node console.

First, execute `yarn install` to install web3 dependencies.

Next, run `./deploy.sh`, which opens a node console. In this console, you can input a combinator contract, and then a holder address (can be any valid Ethereum address except the address sending the contract transaction). A smart contract will be created and the combinator contract will be passed to it, defining its behaviour.

NB: Contract deployment fails on the first attempt per new blockchain, for unknown reasons. Trying a second time will be successful.
