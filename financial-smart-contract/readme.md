## SmartFin Client
This directory contains the financial smart contract (written in Rust), the browser client, and several development tools.

### Prerequisites

In order to run the distributable, you will need python 3 and google chrome. The parity blockchain client should be used for running local blockchains for use with this distributable, or the public Kovan test chain can be used (manually or with MetaMask).

For development, yarn is also required for building the web client.

### Running the SmartFin client

To run the distributable client, run the `dist-client/fsc-server.sh` script. This will run a server with the client build output files, and open a browser tab to the server's URL. This will not initialise a blockchain - to run a local development blockchain, you can execute `./run-node.sh` from this directory. Alternatively, the client can be connected to Kovan.


## Development


### Building the smart contract

To build the smart contract, run `build.sh`. This will output the `contract/target` directory, and copy files to the web client directory.

### Testing the smart contract

To test the smart contract, run `test.sh -rs`. This will run any rust unit tests (defined with `pwasm-test`).

To test the smart contract on the blockchain, run `test.sh -js`. This requires the parity blockchain client to be installed, and will initialise an instance of a local blockchain automatically.

To run both sets of tests, execute `test.sh`.

If running your own separate blockchain instance (with `run-node.sh`), pass `-nc` to `test.sh` (as well as any other options) to prevent the script from initialising its own blockchain, and run tests using the existing blockchain instead (this will be faster if running tests multiple times). The blockchain must be reachable at `localhost:8545`.

### Running the development blockchain

Parity must be installed to run the dev chain. To run the development blockchain (defined in `wasm-dev-chain.json`), execute `run-node.sh`. The blockchain can be cleaned and then run with `run-node.sh --clean`.

*On a fresh local parity blockchain, contract deployment fails on the first attempt per new blockchain. Trying a second time will be successful.*

### Running the financial smart contract client for development

In order to run the financial smart contract client for development, first execute `yarn install` in the client directory (if you haven't already).

Before running the client, the financial smart contract script must be built by running `build.sh`.

After this, run the `run-client.sh` script. This will initialise a parity blockchain, and open the client in dev mode in a browser. To run the client without initialising a blockchain, pass the `-nc` option (as with `test.sh`).

### Building and running the financial smart contract client

To build the financial smart contract client, simply run the `build-client.sh` script. This will build the smart contract, install web dependencies, and compile the SmartFin client distributables in `dist-client`.