import Web3 from "web3";

import { ABI, CODE_HEX } from "./../../resources/resources.mjs";

var web3;

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

    // Returns true/false if this option's value is/isn't defined
    isDefined() {
        return this.defined;
    }

    // Returns the value of this option
    getValue() {
        return this.defined ? this.value : "None";
    }
}

// The observable entry class
export class ObservableEntry {
    // Initialises a new object of this class
    constructor(address, value) {
        this.address = address;
        this.value = new Option(value);
    }

    // Returns the address of the account that this observable entry can be modified by
    getAddress() {
        return this.address;
    }

    // Returns the Optional value of this entry
    getValue() {
        return this.value;
    }
}

// The verification error class.
export class VerificationError {
    /**
     * Initialises a new verification error.
     * @param error The verification error.
     * @param stack The verification error stack.
     */
    constructor(error = "", stack = "") {
        this.error = error;
        this.stack = [stack];
    }
}

export function setupWeb3(url = "http://localhost:8545") {
    var provider = new Web3.providers.HttpProvider(url);
    if (!web3) {
        web3 = new Web3(provider);
    } else {
        web3.setProvider(provider);
    }
    return web3;
}

export function unlockDefaultAccount() {
    if (!web3) {
        setupWeb3();
    }

    web3.eth.defaultAccount = "0x004ec07d2329997267ec62b4166639513386f32e";
    unlockAccount(web3.eth.defaultAccount, "user").then(_ => console.log("Account unlocked"), err => console.log(err));
    return web3.eth.defaultAccount;
}

export async function unlockAccount(address, password) {
    if (!web3) {
        setupWeb3();
    }

    return web3.eth.personal.unlockAccount(address, password, web3.utils.toHex(0));
}

// Loads and deploys the contract (from a fixed contract for this test), returns the contract object
export function loadAndDeployContract(contractBytes, contractHolder, sender) {
    if (!contractBytes || !contractHolder || !sender) {
        return Promise.reject("Expected arguments are contractBytes, contractHolder, and sender. At least one argument was not supplied!");
    }

    if (!web3) {
        setupWeb3();
    }
    
    // Construct a contract object
    var TestContract = new web3.eth.Contract(ABI);
    
    // Construct a deployment transaction
    var TestDeployTransaction = TestContract.deploy({ data: web3.utils.toHex(CODE_HEX), from: sender, arguments: [contractBytes, contractHolder] });
    
    return new Promise(function(resolve, reject) {
        // Attempt to estimate the cost of the deployment transaction
        TestDeployTransaction.estimateGas({}, (err, gas) => {
            if (gas) {
                gas = Math.round(gas * 1.2);
                // Commit the deployment transaction with some extra gas
                TestDeployTransaction.send({ gas: web3.utils.toHex(gas), from: sender }).then(contract => {
                    resolve(contract);
                },
                err => {
                    reject(err);
                })
            } else {
                reject(err)
            }
        });
    })
}

// Serializes a combinator contract from a string
export function serializeCombinatorContract(combinatorContract) {
    if (!web3) {
        setupWeb3();
    }

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
                    var address_serialized = serializeAddress(combinators[i + 2]);
                    for (let part of address_serialized) {
                        result.push(part.toString());
                    }
                    i++;
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

// Verifies a combinator contract, and returns an error message and description if it is ill-formed.
export function verifyContract(contract) {
    if (!contract || contract.length == 0) {
        return {
            error: "No contract given! Please input a combinator contract.",
            description: ""
        };
    }

    var combinators = contract.split(/[ \(\),]/)
        .filter(c => Boolean(c) && c.length != 0)
        .map(c => c.toLowerCase());

    var res = verifyCombinator(combinators, 0);
    if (!res.error && res.endIndex + 1 < combinators.length) {
        res.warning = "This contract contains extraneous combinators after atom " + res.endIndex + ". These will have no effect.";
    }

    return res;
}

// Verifies the combinator at the given index in the given list of combinator atoms.
function verifyCombinator(combinators, i) {
    // Function to generate an error description.
    const errDesc = (index) => {
        return "At: '" + combinators[index] + "', atom: " + index + " of the contract."
    };

    // Function to add a description to an error stack.
    const addToErrStack = (err, index) => {
        err.stack.push(errDesc(index));
        return err;
    }

    if (combinators.length <= i) {
        return new VerificationError("Expected combinator, found end of contract.", "At atom " + i + " of the contract.");
    }

    if (!(combinators[i] in combinatorDict)) {
        return new VerificationError("Expected combinator, found: '" + combinators[i] + "'.", errDesc(i));
    }


    switch (combinators[i]) {
        case "zero":
        case "one":
            // Terminated, return index of termination
            return {
                endIndex: i
            };
        
        case "give":
        case "get":
        case "anytime":
            // One sub-combinator, check it and return index
            var res = verifyCombinator(combinators, i + 1);
            if (res.error) {
                return addToErrStack(res, i);
            } else {
                return res;
            }
        
        case "and":
        case "or":
        case "then":
            // Two sub-combinators, check them and return index
            var res;
            var checked = i;
            for (var j = 0; j < 2; j++) {
                res = verifyCombinator(combinators, checked + 1);

                if (res.error) {
                    return addToErrStack(res, i)
                } else {
                    checked = res.endIndex;
                }
            }
            return res;

        case "truncate":
            // Unix time and sub-combinator, check them and return index
            if (combinators.length <= i + 1) {
                return new VerificationError("Expected a unix timestamp, found end of contract.",errDesc(i));
            }


            // Check timestamp
            var time = combinators[i + 1];
            if (isNaN(time)) {
                return new VerificationError("Expected a valid unix timestamp, found: '" + time + "'.", errDesc(i));
            } else if (combinators[i + 1] < 0 || combinators[i + 1] > Math.pow(2, 32) - 1) {
                // Timestamp is outside of u32 range
                return new VerificationError("Expected unsigned 32-bit unix timestamp, found: '" + time + "'.", errDesc(i));
            }

            // Check sub-combinator
            var res = verifyCombinator(combinators, i + 2);
            if (res.error) {
                return addToErrStack(res, i);
            } else {
                return res;
            }

        case "scale":
            // Observable and address or scale value, and sub-combinator, check them and return index
            var subCombinatorIndex = i + 2;

            if (combinators.length <= i + 1) {
                return new VerificationError("Expected observable or scale value, found end of contract.", errDesc(i));
            }
            var maxValue = BigInt(2) ** BigInt(63);


            if (combinators[i + 1] == "obs") {
                // Observable, check address
                subCombinatorIndex += 1;

                if (combinators.length <= i + 2) {
                    return new VerificationError("Expected observable arbiter address, found end of contract.", errDesc(i));
                }

                var address = combinators[i + 2];
                if (!web3.utils.isAddress(address)) {
                    return new VerificationError("Expected a valid address, found: '" + address + "'.", errDesc(i));
                }
            } else if (isNaN(combinators[i + 1])) {
                return new VerificationError("Expected scale value or 'obs', found: '" + combinators[i + 1] + "'.", errDesc(i));
            } else if (BigInt(combinators[i + 1]) > maxValue - BigInt(1) || BigInt(combinators[i + 1] < -maxValue)) {
                return new VerificationError("Expected signed 64-bit scale value, found: '" + combinators[i + 1] + "'.", errDesc(i));
            }

            var res = verifyCombinator(combinators, subCombinatorIndex);
            if (res.error) {
                return addToErrStack(res, i);
            } else {
                return res;
            }
    }
}

// Serializes an address into 4 integers
export function serializeAddress(address) {
    if (!web3) {
        setupWeb3();
    }

    var bytes = [0,0,0,0].concat(web3.utils.hexToBytes(address));

    var buffer = new ArrayBuffer(32);
    var res = new BigInt64Array(buffer);

    for (var i = 0; i < 3; i++) {
        for (var j = 7; j >= 0; j--) {
            res[i + 1] = res[i + 1] * BigInt(256) + BigInt(bytes[i * 8 + j]);
        }
    }

    return res;
}

// Deserializes 4 integers into an address
export function deserializeAddress(address) {
    if (!web3) {
        setupWeb3();
    }

    var bytes = new Array(20);

    for (var i = 1; i < 4; i++) {
        var block = web3.utils.toBN(address[i].toString());
        // Convert number into bytes
        for (var j = 0; j < 8; j++) {
            // Mod by 256
            var remainder = block.umod(web3.utils.toBN(256));
            bytes[(i - 1) * 8 + j] = remainder;
            // Remove accounted-for byte
            block = block.sub(remainder);
            // Divide by 256
            block = block.div(web3.utils.toBN(256));
        }
    }

    var hex = web3.utils.bytes
    return bytesToAddress(bytes);
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
    if (!web3) {
        setupWeb3();
    }

    orChoices = web3.utils.hexToBytes(orChoices);
    var res = [];

    for (let elem of orChoices) {
        res.push(new Option(elem == 2 ? undefined : elem == 1));
    }

    return res;
}

// Deserializes the observable entries array into an array of ObservableEntries
export function deserializeObsEntries(obsEntries) {
    var res = [];

    for (var i = 0; i < obsEntries.length; i += 5) {
        var address = deserializeAddress(obsEntries.slice(i, i + 4));
        var value = undefined;
        if (obsEntries[i + 4] != -1) {
            value = obsEntries[i + 5];
            i++;
        } 
        res.push(new ObservableEntry(address, value));
    }

    return res;
}

// Converts the given date object to a Unix timestamp
export function dateToUnixTimestamp(date) {
    return date.getTime() / 1000 | 0;
}

// Checks whether or not the given address is valid.
export function isValidAddress(address) {
    if (!web3) {
        setupWeb3();
    }

    return web3.utils.isAddress(address);
}

// Checks whether the account with the given address is a smart contract.
export async function isSmartContract(address) {
    if (!web3) {
        setupWeb3();
    }

    return web3.eth.getCode(address).then(code => {
        if (code == "0x") {
            return Promise.reject("Given address: '" + address + "' corresponds to an externally owned account, not a contract account.");
        } else {
            return Promise.resolve();
        }
    }, _ => {
        return Promise.reject("Given address: '" + address + "' is not a contract account.");
    });
}

// Gets the contract at the given address.
export function getContractAtAddress(address) {
    return new web3.eth.Contract(ABI, address);
}

// Gets the holder of the contract.
export async function getHolder(contract, caller) {
    if (!web3) {
        setupWeb3();
    }

    return contract.methods.get_holder().call({ from: caller }).then(res => {
        return res.returnValue0;
    }, err => {
        return Promise.reject(err);
    });
}

// Gets the counter-party of the contract.
export async function getCounterParty(contract, caller) {
    if (!web3) {
        setupWeb3();
    }

    return contract.methods.get_counter_party().call({ from: caller }).then(res => {
        return res.returnValue0;
    }, err => {
        return Promise.reject(err);
    });
}

// Gets whether or not the contract is concluded.
export async function getConcluded(contract, caller) {
    if (!web3) {
        setupWeb3();
    }

    return contract.methods.get_concluded().call({ from: caller }).then(res => {
        return res.returnValue0;
    }, err => {
        return Promise.reject(err);
    });
}

// Gets the or-choices of the contract.
export async function getOrChoices(contract, caller) {
    if (!web3) {
        setupWeb3();
    }

    return contract.methods.get_or_choices().call({ from: caller }).then(res => {
        return deserializeOrChoices(res.returnValue0);
    }, err => {
        return Promise.reject(err);
    });
}

// Gets the observable entries of the contract.
export async function getObsEntries(contract, caller) {
    if (!web3) {
        setupWeb3();
    }

    return contract.methods.get_obs_entries().call({ from: caller }).then(res => {
        return deserializeObsEntries(res.returnValue0);
    }, err => {
        return Promise.reject(err);
    });
}

// Gets the acquisition times of the given contract.
export async function getAcquisitionTimes(contract, caller) {
    if (!web3) {
        setupWeb3();
    }

    return contract.methods.get_acquisition_times().call({ from: caller }).then(res => {
        return deserializeAcquisitionTimes(res.returnValue0);
    }, err => {
        return Promise.reject(err);
    });
}

// Converts an array of bytes to an address
function bytesToAddress(bytes) {
    if (!web3) {
        setupWeb3();
    }

    var hex = web3.utils.bytesToHex(bytes).substring(2);
    hex = "0x" + hex.padStart(40, "0");

    return web3.utils.toChecksumAddress(hex);
}