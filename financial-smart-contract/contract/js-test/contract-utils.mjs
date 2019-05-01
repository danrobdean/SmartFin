import Web3 from "web3";
import fs from "fs";

export const web3 = new Web3(new Web3.providers.HttpProvider("http://localhost:8545")); // The local parity blockchain address

// Setup combinator -> byte dictionary
const combinatorDict = {
    "zero": 0,
    "one": 1,
    "and": 2,
    "or": 3,
    "truncate": 4,
    "scale": 5,
    "give": 6,
    "then": 7,
    "get": 8,
    "anytime": 9
};

// The Option class
export class Option {
    // Initialises a new object of this class
    constructor(value) {
        this.defined = !!value;
        this.value = value;
    }
}

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
        switch (combinators[i]) {
            case "truncate": {
                result.push(combinator);
                result.push(parseInt(combinators[i + 1]));
                i++;
                break;
            }
            case "scale": {
                result.push(combinator);
                if (combinators[i + 1] != "obs") {
                    result.push(1);
                    result.push(parseInt(combinators[i + 1]));
                } else {
                    result.push(0);
                }
                i++;
                break;
            }
            default:
                result.push(combinator);
                break;
        }
    }

    // Return serialized result as i64 array
    return result;
}

// Serializes an address into 4 integers
export function serializeAddress(address) {
    var bytes = [0,0,0,0].concat(web3.utils.hexToBytes(address));

    var res = new Array(4).fill(0);

    for (var i = 0; i < 3; i++) {
        for (var j = 7; j >= 0; j--) {
            res[i + 1] = web3.utils.toBN(res[i + 1]).mul(web3.utils.toBN(256)).add(web3.utils.toBN(bytes[i * 8 + j]));
        }
    }

    return res;
}

// Deserializes 4 integers into an address
export function deserializeAddress(address) {
    var bytes = new Array(20);

    for (var i = 1; i < 4; i++) {
        var block = address[i];
        for (var j = 0; j < 8; j++) {
            bytes[(i - 1) * 8 + j] = block.umod(web3.utils.toBN(256));
            block = block.div(web3.utils.toBN(256));
        }
    }

    return web3.utils.toChecksumAddress(web3.utils.bytesToHex(bytes));
}

// Deserializes the acquisition times array into an array of Options
export function deserializeAcquisitionTimes(acquisitionTimes) {
    var res = [];

    for (let elem of acquisitionTimes) {
        res.push(new Option(elem == -1 ? undefined : elem));
    }

    return res;
}

// Deserializes the or choices byte array into an array of Options
export function deserializeOrChoices(orChoices) {
    orChoices = web3.utils.hexToBytes(orChoices);
    var res = [];

    for (let elem of orChoices) {
        res.push(new Option(elem == 2 ? undefined : elem == 1));
    }

    return res;
}

// Deserializes the observable values array into an array of Options
export function deserializeObsValues(obsValues) {
    var res = [];

    for (var i = 0; i < obsValues.length; i++) {
        res.push(new Option(obsValues[i] == -1 ? undefined : obsValues[++i]));
    }

    return res;
}


// Loads the contract abi
function getAbi() {
    return JSON.parse(fs.readFileSync("./resources/abi.json"));
}