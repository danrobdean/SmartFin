## Financial Smart Contract
This directory contains the financial smart contract - written in Rust -, the browser client, and several development tools.

### Building the smart contract

To build the smart contract, run `build.sh`. This will output the `contract/target` directory, and copy files to the JS client.

### Testing the smart contract

To test the smart contract, run `test.sh -rs`. This will run any rust unit tests (defined with `pwasm-test`).

To test the smart contract on the blockchain, run `test.sh -js`.

To run both sets of tests, execute `test.sh`.

If running your own blockchain instance (with `run-node.sh`), pass `-nc` to `test.sh` (as well as any other options) to prevent the script from initialising its own blockchain, and run tests using the existing blockchain instead (this will be faster if running tests multiple times).

### Running the development blockchain

Parity must be installed to run the dev chain. To run the development blockchain (defined in `wasm-dev-chain.json`), execute `run-node.sh`. The blockchain can be cleaned and then run with `run-node.sh --clean`.

Once the blockchain is running, the contract can be deployed from a node console.

First, execute `yarn install` in the client directory to install dependencies.

Next, run `./deploy.sh`, which opens a node console. In this console, you can input a combinator contract, and then a holder address (can be any valid Ethereum address except the address sending the contract transaction). A smart contract will be created and the combinator contract will be passed to it, defining its behaviour.

NB: Contract deployment fails on the first attempt per new blockchain, for unknown reasons. Trying a second time will be successful.

### Running the financial smart contract client for development

In order to run the financial smart contract client for development, first execute `yarn install` in the client directory (if you haven't already).

Before running the client, the financial smart contract script must be built by running `build.sh`.

After this, run the `run-client.sh` script. This will initialise a parity blockchain, and open the client in dev mode in a browser. To run the client without initialising a blockchain, pass the `-nc` option (as with `test.sh`).

### Building and running the financial smart contract client

To build the financial smart contract client, simply run the `build-client.sh` script. This will build the smart contract, install web dependencies, and compile the client files.

To run the built client, run the `dist-client/fsc-server.sh` script. This will run a server with the client build output files, and open a browser tab to the server's URL. This will not initialise a blockchain.