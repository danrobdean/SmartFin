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

// Deploy the given combinator contract string
function deploy(contractDefinition) {
    // Get serialized test contract
    var combinatorContract = serializeCombinatorContract(contractDefinition);

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
}

// Get the current UNIX time
function getUnixTime() {
    return Date.now() / 1000 | 0;
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

// Tests for a simple "one" contract
describe('Simple contract tests', function() {
    // Redeploy the smart contract before each test (each deployment has fresh state)
    beforeEach(function() {
        return deploy("one");
    });

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

    it('Updates balances correctly after acquiring', function() {
        // Initial balance is 0
        return contract.methods.get_balance().call({ from: holder.address }).then(function(res) {
            assert.equal(res.returnValue0, 0);
            return contract.methods.get_balance().call({ from: counterParty.address }).then(function(res) {
                assert.equal(res.returnValue0, 0);

                // Acquire the contract
                return contract.methods.acquire().send({ from: holder.address }).then(function() {
                    // New balance for holder is 1
                    return contract.methods.get_balance().call({ from: holder.address }).then(function(res) {
                        assert.equal(res.returnValue0, 1);
                        
                        // New balance for counter-party is -1
                        return contract.methods.get_balance().call({ from: counterParty.address }).then(function(res) {
                            assert.equal(res.returnValue0, -1);
                        });
                    });
                });
            });
        });
    });

    it('Returns concluded only when contract is fully updated', function() {
        return contract.methods.get_concluded().call({ from: holder.address }).then(function(res) {
            assert.ok(!res.returnValue0);

            return contract.methods.acquire().send({ from: holder.address }).then(function() {
                return contract.methods.get_concluded().call({ from: holder.address }).then(function(res) {
                    assert.ok(res.returnValue0);
                });
            });
        });
    });
});

// Tests for an OR contract
describe('OR contract tests', function() {
    it('Has the value of the left sub-combinator when the or choice is set to true', function() {
        return deploy("or one zero").then(function() {
            return contract.methods.set_or_choice(0, true).send({ from: holder.address }).then(function() {
                return contract.methods.acquire().send({ from: holder.address }).then(function() {
                    return contract.methods.get_balance().call({ from: holder.address }).then(function(res) {
                        assert.equal(res.returnValue0, 1);
                    });
                });
            });
        });
    });

    it('Has the value of the right sub-combinator when the or choice is set to false', function() {
        return deploy("or zero one").then(function() {
            return contract.methods.set_or_choice(0, false).send({ from: holder.address }).then(function() {
                return contract.methods.acquire().send({ from: holder.address }).then(function() {
                    return contract.methods.get_balance().call({ from: holder.address }).then(function(res) {
                        assert.equal(res.returnValue0, 1);
                    });
                });
            });
        });
    });
});

// Tests for a TRUNCATE contract
describe('TRUNCATE contract tests', function() {
    it('Is not concluded when the given time is in the past', function() {
        return deploy("truncate " + (getUnixTime() - 100) + " one").then(function() {
            return contract.methods.get_concluded().call({ from: holder.address }).then(function(res) {
                assert.ok(res.returnValue0);
            });
        });
    });

    it('Is concluded when the given time is in the future', function() {
        return deploy("truncate " + (getUnixTime() + 100) + " one").then(function() {
            return contract.methods.get_concluded().call({ from: holder.address }).then(function(res) {
                assert.ok(!res.returnValue0);
            });
        });
    });
});

// Tests for a SCALE contract
describe('SCALE contract tests', function() {
    it('Has the correct value when a scale value is provided', function() {
        return deploy("scale 5 one").then(function() {
            return contract.methods.acquire().send({ from: holder.address }).then(function() {
                return contract.methods.get_balance().call({ from: holder.address }).then(function(res) {
                    assert.equal(res.returnValue0, 5);
                });
            });
        });
    });

    it('Has the correct value when an observable value is provided', function() {
        return deploy("scale obs one").then(function() {
            return contract.methods.propose_obs_value(0, 5).send({ from: counterParty.address }).then(function() {
                return contract.methods.propose_obs_value(0, 5).send({ from: holder.address }).then(function() {
                    return contract.methods.acquire().send({ from: holder.address }).then(function() {
                        return contract.methods.get_balance().call({ from: holder.address }).then(function(res) {
                            assert.equal(res.returnValue0, 5);
                        });
                    });
                });
            });
        });
    });
});

// Tests for an ANYTIME contract
describe('ANYTIME contract tests', function() {
    it('Has the correct value before the anytime sub-combinator is acquired', function() {
        return deploy("anytime one").then(function() {
            return contract.methods.acquire().send({ from: holder.address }).then(function() {
                return contract.methods.get_balance().call({ from: holder.address }).then(function(res) {
                    assert.equal(res.returnValue0, 0);
                });
            });
        })
    });

    it('Has the correct value after the anytime sub-combinator is acquired', function() {
        return deploy("anytime one").then(function() {
            return contract.methods.acquire().send({ from: holder.address }).then(function() {
                return contract.methods.acquire_anytime_sub_contract(0).send({ from: holder.address }).then(function() {
                    return contract.methods.get_balance().call({ from: holder.address }).then(function(res) {
                        assert.equal(res.returnValue0, 1);
                    });
                });
            });
        })
    });
});

// Miscellaneous tests
describe('Miscellaneous tests', function() {
    it('Returns the correct serialized contract', function() {
        let contractDefinition = "and or one zero truncate 100 get give anytime then scale 500 one zero";
        let serializedContract = serializeCombinatorContract(contractDefinition);
        return deploy(contractDefinition).then(function() {
            return contract.methods.get_contract_definition().call({ from: holder.address }).then(function(res) {
                assert.deepEqual(res.returnValue0, serializedContract);
            });
        });
    }).timeout(5000);
});