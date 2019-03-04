// Set-up and function definition mainly for deploying the test contract from a node console (and some other util functions)

// Load modules
var Web3 = require("web3");
var web3 = new Web3(new Web3.providers.HttpProvider("http://localhost:8545"));
var fs = require("fs");

// Set default account for transactions (this is pre-defined on our testing blockchain)
web3.eth.defaultAccount = "0x004ec07d2329997267ec62b4166639513386f32e";

// Loads the abi (from a fixed location for this test)
function getAbi() {
    return JSON.parse(fs.readFileSync("./target/json/FinancialScInterface.json"));
}

// Unlocks an account (pre-defined on our testing blockchain)
function unlockAccount() {
    web3.eth.personal.unlockAccount(web3.eth.defaultAccount, "user", web3.utils.toHex(0)).then(_ => console.log("Account unlocked"), err => console.log(err));
}

// Loads and deploys the contract (from a fixed contract for this test), returns the contract object
function loadAndDeployContract() {
    var abi = getAbi();

    // Format the contract correctly
    var codeHex = '0x' + fs.readFileSync("./target/financial_smart_contract.wasm").toString('hex');
    
    // Construct a contract object
    var TestContract = new web3.eth.Contract(abi);
    
    // Construct a deployment transaction
    var TestDeployTransaction = TestContract.deploy({ data: codeHex, from: web3.eth.defaultAccount });
    
    // Attempt to estimate the cost of the deployment transaction
    TestDeployTransaction.estimateGas({}, (err, gas) => {
        if (gas) {
            console.log("Gas: " + gas + "\n");
            gas = Math.round(gas * 1.2);
            // Commit the deployment transaction with some extra gas
            TestDeployTransaction.send({ gas: web3.utils.toHex(gas), from: web3.eth.defaultAccount }).then(contract => {
                console.log("Contract deployed at: " + contract.options.address);
                TestContract = contract;
                return TestContract;
            },
            err => {
                console.log("Error deploying contract:\n" + err);
            })
        } else {
            console.log("Error estimating gas:\n" + err);
        }
    });
}

// Prints the latest block
function getBlock() {
    return web3.eth.getBlock("latest", (_, block) => {console.log(block)});
}

// Gets a contract object from the tx receipt
function getContractFromTx(receipt) {
    return web3.eth.getTransactionReceipt(receipt, (_, tx) => {console.log(tx.contractAddress)});
}

// Gets a contract object from the given address
function getContract(address) {
    address = web3.utils.toHex(address);
    var contract = new web3.eth.Contract(getAbi(), "address", {});
    return contract;
}
