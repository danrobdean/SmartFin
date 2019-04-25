import Web3 from "web3";
import fs from "fs";

const web3 = new Web3(new Web3.providers.HttpProvider("http://localhost:8545")); // The local parity blockchain address

// Setup combinator -> byte dictionary
const combinatorDict = {
    "zero": 0,
    "one": 1,
    "and": 2,
    "or": 3
};

export function unlockDefaultAccount() {
    web3.eth.defaultAccount = "0x004ec07d2329997267ec62b4166639513386f32e";
    unlockAccount(web3.eth.defaultAccount, "user").then(_ => console.log("Account unlocked"), err => console.log(err));
}

export async function unlockAccount(address, password) {
    return web3.eth.personal.unlockAccount(address, password, web3.utils.toHex(0));
}

// Loads and deploys the contract (from a fixed contract for this test), returns the contract object
export function loadAndDeployContract(contractBytes, contractHolder, sender = "0x004ec07d2329997267ec62b4166639513386f32e") {
    var abi = getAbi();

    // Format the contract correctly
    var codeHex = '0x' + fs.readFileSync("./resources/contract.wasm").toString('hex');
    
    // Construct a contract object
    var TestContract = new web3.eth.Contract(abi);
    
    // Construct a deployment transaction
    var TestDeployTransaction = TestContract.deploy({ data: codeHex, from: sender, arguments: [contractBytes, contractHolder] });
    
    return new Promise(function(resolve, reject) {
        // Attempt to estimate the cost of the deployment transaction
        TestDeployTransaction.estimateGas({}, (err, gas) => {
            if (gas) {
                gas = Math.round(gas * 1.2);
                // Commit the deployment transaction with some extra gas
                TestDeployTransaction.send({ gas: web3.utils.toHex(gas), from: sender }).then(contract => {
                    console.log("Contract deployed at: " + contract.options.address);
                    resolve(contract);
                },
                err => reject(err))
            } else {
                reject(err)
            }
        });
    })
}

// Serializes a combinator contract from a string
export function serializeCombinatorContract(combinatorContract) {
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

    // Return serialized result as i64 array
    return result;
}


// Loads the contract abi
function getAbi() {
    return JSON.parse(fs.readFileSync("./resources/abi.json"));
}