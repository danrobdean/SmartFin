import assert from "assert";
import { unlockAccount, loadAndDeployContract, serializeCombinatorContract } from "./contract-utils.mjs";

var contract;

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
before(function(done) {
    // Unlock accounts
    unlockAccount(holder.address, holder.password).then(function(_) {
        unlockAccount(counterParty.address, counterParty.password).then(function(_) {
            unlockAccount(uninvolved.address, uninvolved.password).then(function(_) {
                // Get serialized test contract
                var combinatorContract = serializeCombinatorContract("one");
    
                // First deployment may fail
                loadAndDeployContract(combinatorContract, holder.address, counterParty.address).then(function(res) {
                    // First deployment succeeded
                    contract = res;
                    done();
                }, function(_) {
                    // Should deploy successfully
                    loadAndDeployContract(combinatorContract, holder.address, counterParty.address).then(function(res) {
                        contract = res;
                        done();
                    }, function(err) {
                        done(err);
                    });
                });
            });
        });
    });
});

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
        return contract.methods.stake().send({ from: holder.address, value: 100 }).then(function() {
            return contract.methods.get_balance().call({ from: holder.address }).then(function(res) {
                assert.equal(res.returnValue0, 100);
            });
        });
    });

    it('Updates stake correctly for the counter-party', function() {
        return contract.methods.stake().send({ from: counterParty.address, value: 100 }).then(function() {
            return contract.methods.get_balance().call({ from: counterParty.address }).then(function(res) {
                assert.equal(res.returnValue0, 100);
            });
        });
    });
})