import assert from "assert";
import { web3, unlockAccount, loadAndDeployContract, serializeCombinatorContract } from "./contract-utils.mjs";

// The deployed smart contract instance
var contract;

// Address/password pairs
const uninvolved = {
    address: "0x004ec07d2329997267ec62b4166639513386f32e",
    password: "user"
}

const holder = {
    address: "0x057E231DaB35A789F5999056c8Ec775512609CBb",
    password: "test"
};

const counterParty = {
    address: "0x1e00c1c4f7c9C878e863E9B2acC374F0C2a0F742",
    password: "test"
}

// Testing hooks

// Unlock accounts before all tests
before(function() {
    // Unlock accounts
    return unlockAccount(holder.address, holder.password).then(function() {
        unlockAccount(counterParty.address, counterParty.password).then(function() {
            unlockAccount(uninvolved.address, uninvolved.password);
        });
    });
});

// Redeploy the smart contract before each test (each deployment has fresh state)
beforeEach(function() {
    // Get serialized test contract
    var combinatorContract = serializeCombinatorContract("one");

    // First deployment may fail
    return loadAndDeployContract(combinatorContract, holder.address, counterParty.address).then(function(res) {
        // First deployment succeeded
        contract = res;
    }, function(_) {
        // Should deploy successfully
        return loadAndDeployContract(combinatorContract, holder.address, counterParty.address).then(function(res) {
            contract = res;
        });
    });
});

// Tests
describe('Contract tests', function() {
    it('Returns the correct holder address', function() {
        return contract.methods.get_holder().call({ from: uninvolved.address }).then(function(res) {
            assert.equal(res.returnValue0, holder.address);
        });
    });

    it('Returns the correct counter-party address', function() {
        return contract.methods.get_counter_party().call({ from: uninvolved.address }).then(function(res) {
            assert.equal(res.returnValue0, counterParty.address);
        });
    });

    it('Updates stake correctly for the holder', function() {
        var stake = 100;

        return contract.methods.stake().send({ from: holder.address, value: stake }).then(function() {
            return contract.methods.get_balance().call({ from: holder.address }).then(function(res) {
                assert.equal(stake, res.returnValue0);
            });
        });
    });

    it('Updates stake correctly for the counter-party', function() {
        var stake = 100;

        return contract.methods.stake().send({ from: counterParty.address, value: stake }).then(function() {
            return contract.methods.get_balance().call({ from: counterParty.address }).then(function(res) {
                assert.equal(stake, res.returnValue0);
            });
        });
    });

    it('Pays the right amount when withdrawing for the holder', function() {
        var stake = 1000000;
        var withdrawal = stake / 10;

        return web3.eth.getBalance(holder.address).then(function(balance) {
            return contract.methods.stake().send({ from: holder.address, value: stake }).then(function() {
                return contract.methods.withdraw(withdrawal).send({ from: holder.address }).then(function() {
                    return web3.eth.getBalance(holder.address).then(function(newBalance) {
                        var balanceBN = web3.utils.toBN(balance);
                        var newBalanceBN = web3.utils.toBN(newBalance);
                        var stakeBN = web3.utils.toBN(stake);
                        var withdrawalBN = web3.utils.toBN(withdrawal);

                        assert.equal(newBalanceBN.toString(), balanceBN.sub(stakeBN).add(withdrawalBN).toString());
                    });
                });
            });
        });
    });

    it('Pays the right amount when withdrawing for the counter-party', function() {
        var stake = 1000000;
        var withdrawal = stake / 10;

        return web3.eth.getBalance(counterParty.address).then(function(balance) {
            return contract.methods.stake().send({ from: counterParty.address, value: stake }).then(function() {
                return contract.methods.withdraw(withdrawal).send({ from: counterParty.address }).then(function() {
                    return web3.eth.getBalance(counterParty.address).then(function(newBalance) {
                        var balanceBN = web3.utils.toBN(balance);
                        var newBalanceBN = web3.utils.toBN(newBalance);
                        var stakeBN = web3.utils.toBN(stake);
                        var withdrawalBN = web3.utils.toBN(withdrawal);

                        assert.equal(newBalanceBN.toString(), balanceBN.sub(stakeBN).add(withdrawalBN).toString());
                    });
                });
            });
        });
    });
})