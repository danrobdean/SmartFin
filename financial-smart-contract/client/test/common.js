import { setupWeb3, loadAndDeployContract, serializeCombinatorContract, dateToUnixTimestamp, unlockAccount } from "./../src/js/contract-utils.mjs";

// The deployed smart contract instance
export var web3 = setupWeb3();

// Address/password pairs
export const uninvolved = {
    address: web3.utils.toChecksumAddress("0x37aC31b396F68051e2a5D148CaF2198Af45ac918"),
    password: "test"
}

export const holder = {
    address: web3.utils.toChecksumAddress("0x057E231DaB35A789F5999056c8Ec775512609CBb"),
    password: "test"
};

export const counterParty = {
    address: web3.utils.toChecksumAddress("0x1e00c1c4f7c9C878e863E9B2acC374F0C2a0F742"),
    password: "test"
}

// Deploy the given combinator contract string
export function deploy(contractDefinition, useGas=true) {
    // Get serialized test contract
    var combinatorContract = serializeCombinatorContract(contractDefinition);

    // First deployment may fail
    return loadAndDeployContract(combinatorContract, holder.address, counterParty.address, useGas).then(function(res) {
        // First deployment succeeded
        return res;
    }, function(_) {
        // Should deploy successfully
        return loadAndDeployContract(combinatorContract, holder.address, counterParty.address, useGas).then(function(res) {
            return res;
        });
    });
}

// Get the current UNIX time
export function getUnixTime() {
    return dateToUnixTimestamp(new Date());
}

// Unlock accounts before all tests
export function unlockAccounts() {
    // Unlock accounts
    return unlockAccount(holder.address, holder.password).then(function() {
        return unlockAccount(counterParty.address, counterParty.password).then(function() {
            return unlockAccount(uninvolved.address, uninvolved.password);
        });
    });
}

// Unlock accounts before all tests.
before(function() {
    unlockAccounts();
});