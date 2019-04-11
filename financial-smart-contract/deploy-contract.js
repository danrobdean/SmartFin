// Set-up and function definition mainly for deploying the test contract from a node console (and some other util functions)

// Load modules
var Web3 = require("web3");
var web3 = new Web3(new Web3.providers.HttpProvider("http://localhost:8545"));
var fs = require("fs");
const readline = require("readline");

// Setup readline
const r1 = readline.createInterface({
    input: process.stdin,
    output: process.stdout
});

// Setup combinator -> byte dictionary
const combinatorDict = {
    "zero": 0,
    "one": 1,
    "and": 2
};


// Loads the abi (from a fixed location for this test)
function getAbi() {
    return JSON.parse(fs.readFileSync("./target/json/FinancialScInterface.json"));
}

// Unlocks an account (pre-defined on our testing blockchain)
function unlockAccount() {
    web3.eth.personal.unlockAccount(web3.eth.defaultAccount, "user", web3.utils.toHex(0)).then(_ => console.log("Account unlocked"), err => console.log(err));
}

// Loads and deploys the contract (from a fixed contract for this test), returns the contract object
function loadAndDeployContract(contractBytes) {
    var abi = getAbi();

    // Format the contract correctly
    var codeHex = '0x' + fs.readFileSync("./target/financial_smart_contract.wasm").toString('hex');
    
    // Construct a contract object
    var TestContract = new web3.eth.Contract(abi);
    
    // Construct a deployment transaction
    var TestDeployTransaction = TestContract.deploy({ data: codeHex, from: web3.eth.defaultAccount, arguments: [contractBytes] });
    
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

// Serializes a combinator contract from a string
function serializeCombinatorContract(combinatorContract) {
    var combinators = combinatorContract.split(/[ \(\),]/)
        .filter(c => c.length != 0)
        .map(c => c.toLowerCase());
    var result = [];
    for (var i = 0; i < combinators.length; i++) {
        // Lookup value of combinator when serialized
        var combinator = combinatorDict[combinators[i]];
        if (combinator == undefined) {
            throw "Combinator " + combinators[i] + " not recognized.";
        }

        // Add combinator values to serialized result
        switch (combinator) {
            default:
                result.push(combinator);
                break;
        }
    }

    console.log("Serialized combinator contract: [" + result + "]");

    // Return serialized result as Uint8Array (byte array)
    return Uint8Array.from(result);
}

// Obtain contract from IO, then handle
r1.question("Please input a combinator contract, or press ENTER to exit: ", (answer) => {
    var combinatorContract = answer.trim();

    // Close if no contract entered
    if (combinatorContract == "") {
        r1.close();
        return;
    }

    // Set default account for transactions (this is pre-defined on our testing blockchain) and unlock
    web3.eth.defaultAccount = "0x004ec07d2329997267ec62b4166639513386f32e";
    unlockAccount();

    // Serialize contract
    var serializedCombinatorContract = serializeCombinatorContract(combinatorContract);

    // Deploy contract
    loadAndDeployContract(serializedCombinatorContract);

    r1.close();
});