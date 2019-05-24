import Web3 from "web3";

import { ABI, CODE_HEX } from "./../../resources/resources.mjs";

// Returns an object with the given object's keys and values inverted
function invert(obj) {
    var inverted = {};

    for (var key in obj) {
        if (obj.hasOwnProperty(key)) {
            inverted[obj[key]] = key;
        }
    }

    return inverted;
}

// Initialise the web3 instance
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

const serializedCombinatorDict = invert(combinatorDict);

// The Option class
export class Option {
    // Initialises a new object of this class
    constructor(value) {
        this.defined = value !== undefined;
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
    constructor(address, value, name, index) {
        this.address = address;
        this.value = new Option(value);
        this.name = name;
        this.index = index;
    }

    // Returns the address of the account that this observable entry can be modified by
    getAddress() {
        return this.address;
    }

    // Returns the Optional value of this entry
    getValue() {
        return this.value;
    }

    // Returns the name of this observable
    getName() {
        return this.name;
    }

    // Returns the index of this observable
    getIndex() {
        return this.index;
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

// Class for representing the result of contract deserialization.
export class DeserializeResult {
    /**
     * Initialises a new instance of this class.
     * @param contract The contract string.
     * @param endIndex The index in the serialized combinator array that this contract reaches up to.
     */
    constructor(contract, endIndex) {
        this.contract = contract;
        this.endIndex = endIndex;
    }

    // Gets the contract string.
    getContract() {
        return this.contract;
    }

    // Gets the end index.
    getEndIndex() {
        return this.endIndex;
    }
}

export function setupWeb3(metamask = false, url = "http://localhost:8545") {
    var provider;
    if (metamask && typeof window.web3 !== undefined) {
        provider = window.web3.currentProvider;
    } else {
        provider = new Web3.providers.HttpProvider(url);
    }

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
export function loadAndDeployContract(contractBytes, contractHolder, sender, useGas) {
    if (!contractBytes || !contractHolder || !sender) {
        return Promise.reject("Expected arguments are contractBytes, contractHolder, and sender. At least one argument was not supplied!");
    }

    if (!web3) {
        setupWeb3();
    }
    
    // Construct a contract object
    var TestContract = new web3.eth.Contract(ABI);
    
    // Construct a deployment transaction
    var TestDeployTransaction = TestContract.deploy({ data: web3.utils.toHex(CODE_HEX), from: sender, arguments: [contractBytes, contractHolder, useGas] });
    
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
        .filter(c => c.length != 0);
    var result = [];
    for (var i = 0; i < combinators.length; i++) {
        // Lookup value of combinator when serialized
        var combinator = combinatorDict[combinators[i]];
        if (combinator == undefined) {
            throw "Combinator " + combinators[i] + " not recognized.";
        }

        // Add combinator values to serialized result
        switch (combinators[i].toLowerCase()) {
            case "truncate": {
                result.push(combinator);
                result.push(parseInt(combinators[i + 1]));
                i++;
                break;
            }
            case "scale": {
                result.push(combinator);
                if (!isNaN(combinators[i + 1])) {
                    // Scale value, push 1
                    result.push(1);

                    // Push scale value
                    result.push(parseInt(combinators[i + 1]));

                    i += 1;
                } else {
                    // Observable, push 0
                    result.push(0);

                    // Push address
                    var addressSerialized = serializeAddress(combinators[i + 2]);
                    for (let part of addressSerialized) {
                        result.push(part.toString());
                    }

                    // Push observable name
                    var nameSerialized = serializeName(combinators[i + 1]);
                    for (let part of nameSerialized) {
                        result.push(part.toString());
                    }

                    i += 2;
                }
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
    if (!web3) {
        setupWeb3();
    }

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


            if (isNaN(combinators[i + 1])) {
                // Observable, check address
                subCombinatorIndex += 1;

                if (combinators.length <= i + 2) {
                    return new VerificationError("Expected observable arbiter address, found end of contract.", errDesc(i));
                }

                var address = combinators[i + 2];
                if (!web3.utils.isAddress(address)) {
                    return new VerificationError("Expected a valid address, found: '" + address + "'.", errDesc(i));
                }
            } else if (!isValidScaleValue(combinators[i + 1])) {
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

    return bytesToAddress(bytes);
}

// Deserializes the acquisition times array into an array of Options
export function deserializeAcquisitionTimes(acquisitionTimes) {
    var res = [];

    if (acquisitionTimes) {
        for (let elem of acquisitionTimes) {
            res.push(new Option(elem == -1 ? undefined : elem));
        }
    }

    return res;
}

// Deserializes the or choices byte array into an array of Options
export function deserializeOrChoices(orChoices) {
    if (!web3) {
        setupWeb3();
    }

    var res = [];
    if (orChoices) {
        orChoices = web3.utils.hexToBytes(orChoices);

        for (let elem of orChoices) {
            res.push(new Option(elem >= 2 ? undefined : elem == 1));
        }
    }

    return res;
}

// Deserializes the observable entries array into an array of ObservableEntries
export function deserializeObsEntries(obsEntries) {
    var res = [];
    var nameLen = 0;
    var index = 0;

    if (obsEntries) {
        for (var i = 0; i < obsEntries.length; i += 6 + nameLen) {
            var address = deserializeAddress(obsEntries.slice(i, i + 4));

            var value = undefined;
            if (obsEntries[i + 4] != -1) {
                value = obsEntries[i + 5];
                i++;
            }

            nameLen = parseInt(obsEntries[i + 5]);
            var name = deserializeName(obsEntries.slice(i + 6, i + 6 + nameLen));

            res.push(new ObservableEntry(address, value, name, index++));
        }
    }

    return res;
}

// Converts a name string into an array ([N, char0, char1..., charN])
export function serializeName(name) {
    var res = Array.from(name).map(c => c.charCodeAt(0));
    res.unshift(name.length);

    return res;
}

// Converts a serialized name array ([N, char0, char1..., charN]) into a string
export function deserializeName(nameSerialized) {
    return String.fromCharCode(...nameSerialized);
}

// Converts the given date object to a Unix timestamp
export function dateToUnixTimestamp(date) {
    return date.getTime() / 1000 | 0;
}

// Converts the given unix timestamp to a date string.
export function unixTimestampToDateString(timestamp) {
    var date = new Date(timestamp * 1000);
    var options = {
        year: "numeric",
        month: "2-digit",
        day: "2-digit",
        hour: "2-digit",
        minute: "2-digit",
        second: "2-digit"
    }
    
    return Intl.DateTimeFormat("en-GB", options).format(date);
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

// Gets whether or not the contract allocates gas fees upon withdrawal.
export async function getUseGas(contract, caller) {
    if (!web3) {
        setupWeb3();
    }

    return contract.methods.get_use_gas().call({ from: caller }).then(res => {
        return res.returnValue0;
    }, err => {
        return Promise.reject(err);
    });
}

// Gets the last-updated time.
export async function getLastUpdated(contract, caller) {
    if (!web3) {
        setupWeb3();
    }

    return contract.methods.get_last_updated().call({ from: caller }).then(res => {
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

// Sets the or choice of an or combinator on the given contract.
export async function setOrChoice(contract, caller, index, choice) {
    if (!web3) {
        setupWeb3();
    }

    return contract.methods.set_or_choice(index, choice).send({ from: caller }).catch(err => {
        return Promise.reject(err);
    });
}

// Sets the value of an observable.
export async function setObsValue(contract, caller, index, value) {
    if (!web3) {
        setupWeb3();
    }

    if (!isValidScaleValue(value)) {
        return Promise.reject("The given value is not a valid 64-bit signed integer.")
    }

    return contract.methods.set_obs_value(index, value).send({ from: caller }).catch(err => {
        return Promise.reject(err);
    });
}

// Acquires the contract.
export async function acquireContract(contract, caller) {
    if (!web3) {
        setupWeb3();
    }

    return contract.methods.acquire().send({ from: caller }).catch(err => {
        return Promise.reject(err);
    });
}

// Acquires a sub-contract.
export async function acquireSubContract(contract, caller, index) {
    if (!web3) {
        setupWeb3();
    }

    return contract.methods.acquire_anytime_sub_contract(index).send({ from: caller }).catch(err => {
        return Promise.reject(err);
    });
}

// Updates the contract (costs gas).
export async function updateContract(contract, caller) {
    if (!web3) {
        setupWeb3();
    }

    return contract.methods.update().send({ from: caller }).catch(err => {
        return Promise.reject(err);
    });
}

// Gets the balance of the given party (true is holder, false counter-party)
export async function getBalance(contract, caller, holderBalance) {
    if (!web3) {
        setupWeb3();
    }

    return contract.methods.get_balance(holderBalance).call({ from: caller }).then(res => {
        return res.returnValue0;
    }, err => {
        return Promise.reject(err);
    });
}

// Stakes the given amount of wei to the given contract.
export async function stake(contract, caller, amount) {
    if (!web3) {
        setupWeb3();
    }

    return contract.methods.stake().send({ from: caller, value: amount }).catch(err => {
        return Promise.reject(err);
    });
}

// Withdraws the given amount of wei from the given contract.
export async function withdraw(contract, caller, amount) {
    if (!web3) {
        setupWeb3();
    }

    return contract.methods.withdraw(amount).send({ from: caller }).catch(err => {
        return Promise.reject(err);
    });
}

// Deserializes the serialized combinator contract definition.
export function deserializeCombinatorContract(i, serializedCombinatorContract) {
    if (!serializedCombinatorContract || serializedCombinatorContract.length == 0) {
        throw "Attempted to deserialize invalid combinator contract.";
    }

    var combinator = serializedCombinatorDict[serializedCombinatorContract[i]];

    switch (combinator) {
        case "zero":
        case "one": {
            return new DeserializeResult(combinator, i);
        }

        case "and":
        case "or":
        case "then": {
            let contract = combinator + " ";

            let subRes0 = deserializeCombinatorContract(i + 1, serializedCombinatorContract);
            let subRes1 = deserializeCombinatorContract(subRes0.getEndIndex() + 1, serializedCombinatorContract);

            contract += subRes0.getContract() + " " + subRes1.getContract();
            return new DeserializeResult(contract, subRes1.getEndIndex());
        }

        case "give":
        case "get":
        case "anytime": {
            let contract = combinator + " ";

            let subRes = deserializeCombinatorContract(i + 1, serializedCombinatorContract);

            contract += subRes.getContract();
            return new DeserializeResult(contract, subRes.getEndIndex());
        }

        case "truncate": {
            let contract = combinator + " <";
            contract += unixTimestampToDateString(serializedCombinatorContract[i + 1]) + "> ";

            let subRes = deserializeCombinatorContract(i + 2, serializedCombinatorContract);

            contract += subRes.getContract();
            return new DeserializeResult(contract, subRes.getEndIndex());
        }

        case "scale": {
            let contract = combinator + " ";
            var nextIndex;

            if (serializedCombinatorContract[i + 1] == 0) {
                var address = deserializeAddress(serializedCombinatorContract.slice(i + 2, i + 6));

                var nameLen = parseInt(serializedCombinatorContract[i + 6]);
                var name = deserializeName(serializedCombinatorContract.slice(i + 7, i + 7 + nameLen));

                contract += name + " <" + address + "> ";
                nextIndex = i + 7 + nameLen;
            } else if (serializedCombinatorContract[i + 1] == 1) {
                contract += serializedCombinatorContract[i + 2] + " ";
                nextIndex = i + 3;
            }

            let subRes = deserializeCombinatorContract(nextIndex, serializedCombinatorContract);

            contract += subRes.getContract() ;
            return new DeserializeResult(contract, subRes.getEndIndex());
        }
            
        default:
            throw "Unknown combinator found in contract definition."
    }
}

// Gets the combinator contract definition from the given contract.
export async function getCombinatorContract(contract, caller) {
    if (!web3) {
        setupWeb3();
    }

    return contract.methods.get_contract_definition().call({ from: caller}).then(res => {
        try {
            return deserializeCombinatorContract(0, res.returnValue0).getContract();
        } catch (err) {
            return Promise.reject(err);
        }
    }, err => {
        return Promise.reject(err);
    });
}

// Returns true if the given value can be used as a scale value, false otherwise
export function isValidScaleValue(value) {
    if (!web3) {
        setupWeb3();
    }
    if (isNaN(value)) {
        return false;
    }

    var maxValue = web3.utils.toBN(2).pow(web3.utils.toBN(63));
    var valueBN = web3.utils.toBN(value);
    // Check is number and in bounds
    return !(valueBN.gt(maxValue.sub(web3.utils.toBN(1))) || valueBN.lt(maxValue.neg()));
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