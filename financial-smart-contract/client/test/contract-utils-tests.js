import assert from "assert";

import { holder, uninvolved, counterParty, deploy, unlockAccounts } from "./common";
import * as Utils from "../src/js/contract-utils.mjs";


describe('Contract utility tests', function() {
    describe('Utility function tests', function() {
        it('Correctly serializates/deserializes address', function() {
            var address = uninvolved.address;
            var serialized = Utils.serializeAddress(address);
            var deserialized = Utils.deserializeAddress(serialized);
            assert.equal(deserialized, address);
        });

        it('Correctly deserializes acquisition times', function() {
            var acquisitionTimesSerialized = [10, -1, 1000];
            var expectedDeserialized = [
                new Utils.Option(10),
                new Utils.Option(undefined),
                new Utils.Option(1000)
            ];
            
            assert.deepEqual(Utils.deserializeAcquisitionTimes(acquisitionTimesSerialized), expectedDeserialized);
        });

        it('Correctly deserializes or-choices', function() {
            var orChoices = "0x010002";
            var expectedDeserialized = [
                new Utils.Option(true),
                new Utils.Option(false),
                new Utils.Option(undefined)
            ];

            assert.deepEqual(Utils.deserializeOrChoices(orChoices), expectedDeserialized);
        });

        it('Correctly deserializes obs-entries', function() {
            var names = ["name0", "namEe1", "name2"];
            var obsEntries = [
                ...Utils.serializeAddress(uninvolved.address), 0, 10, names[0].length, ...Array.from(names[0]).map(c => c.charCodeAt(0)),
                ...Utils.serializeAddress(holder.address), 0, -100, names[1].length, ...Array.from(names[1]).map(c => c.charCodeAt(0)),
                ...Utils.serializeAddress(counterParty.address), -1, names[2].length, ...Array.from(names[2]).map(c => c.charCodeAt(0))
            ];
            var expectedDeserialized = [
                new Utils.ObservableEntry(uninvolved.address, 10, names[0], 0),
                new Utils.ObservableEntry(holder.address, -100, names[1], 1),
                new Utils.ObservableEntry(counterParty.address, undefined, names[2], 2)
            ];

            assert.deepEqual(Utils.deserializeObsEntries(obsEntries), expectedDeserialized);
        });

        it('Correctly serializes names', function() {
            var name = "0nameTesT123";
            var expectedName = Array.from(name).map(c => c.charCodeAt(0));
            expectedName.unshift(name.length);

            assert.deepEqual(Utils.serializeName(name), expectedName);
        });

        it('Correctly deserializes names', function() {
            var name = "0nameTesT123";
            var nameDeserialized = Utils.serializeName(name);
            nameDeserialized.shift();

            assert.equal(Utils.deserializeName(nameDeserialized), name);
        });

        it('Correctly converts a date to/from a unix timestamp', function() {
            var testDates = [
                new Date("1973-11-29 21:33:09 GMT"),
                new Date("2001-04-19 04:25:21 GMT"),
                new Date("2001-09-09 01:46:40 GMT"),
                new Date("2038-01-19 00:00:00 GMT")
            ];
            var testUnix = [
                123456789,
                987654321,
                1000000000,
                2147472000
            ];

            for (var i = 0; i < testDates.length; i++) {
                assert.equal(Utils.dateToUnixTimestamp(testDates[i]), testUnix[i]);
                assert.equal(new Date(Utils.unixTimestampToDateString(testUnix[i])).toString(), testDates[i].toString());
            }
        });
        
        it('Correctly validates an address', function() {
            assert.equal(Utils.isValidAddress(uninvolved.address), true);
            assert.equal(Utils.isValidAddress("0x123456789"), false);
        });

        it('Correctly checks whether an address is a smart contract or not', function() {
            return unlockAccounts().then(function() {
                return Utils.isSmartContract(uninvolved.address).then(function() {
                    assert.fail();
                }, function() {
                    return deploy("one").then(function(contract) {
                        Utils.isSmartContract(contract.address).catch(function() {
                            assert.fail();
                        });
                    }, function() {
                        assert.fail();
                    });
                });
            });
        });

        it('Correctly gets the contract at the given address', function() {
            return unlockAccounts().then(function() {
                return deploy("one", false).then(contract => {
                    assert.equal(Utils.getContractAtAddress(contract.address).address, contract.address);
                });
            });
        });

        it('Correctly validates a scale value', function() {
            assert.equal(Utils.isValidScaleValue(123456789), true);
            assert.equal(Utils.isValidScaleValue(-100), true);
            assert.equal(Utils.isValidScaleValue("test"), false);
            assert.equal(Utils.isValidScaleValue("9223372036854775807"), true);
            assert.equal(Utils.isValidScaleValue("9223372036854775808"), false);
            assert.equal(Utils.isValidScaleValue("-9223372036854775808"), true);
            assert.equal(Utils.isValidScaleValue("-9223372036854775809"), false);
        });
    });

    describe('Contract serialization/deserialization tests', function() {
        it('Correctly serializes basic smart contracts', function() {
            assert.deepEqual(Utils.serializeCombinatorContract("zero"), [0]);
            assert.deepEqual(Utils.serializeCombinatorContract("one"), [1]);
        });

        it('Correctly serializes a give combinator', function() {
            assert.deepEqual(Utils.serializeCombinatorContract("give one"), [6, 1]);
        });

        it('Correctly serializes a get combinator', function() {
            assert.deepEqual(Utils.serializeCombinatorContract("get one"), [8, 1]);
        });

        it('Correctly serializes a scale combinator with a scale value', function() {
            assert.deepEqual(Utils.serializeCombinatorContract("scale 123 one"), [5, 1, 123, 1]);
        });

        it('Correctly serializes a scale combinator with an observable', function() {
            var name = "namE123"
            var contractSerialized = Utils.serializeCombinatorContract("scale " + name + " " + uninvolved.address + " one");
            var expectedSerialized = [5, 0, ...Utils.serializeAddress(uninvolved.address), ...Utils.serializeName(name), 1];
            assert.deepEqual(contractSerialized, expectedSerialized);
        });

        it('Correctly serializes a truncate combinator', function() {
            var time = 123456789;
            var contractSerialized = Utils.serializeCombinatorContract("truncate " + time + " one");
            var expectedSerialized = [4, time, 1];
            assert.deepEqual(contractSerialized, expectedSerialized);
        });

        it('Correctly serializes an anytime combinator', function() {
            assert.deepEqual(Utils.serializeCombinatorContract("anytime one"), [9, 1]);
        });

        it('Correctly serializes an and combinator', function() {
            assert.deepEqual(Utils.serializeCombinatorContract("and zero one"), [2, 0, 1]);
        });

        it('Correctly serializes an or combinator', function() {
            assert.deepEqual(Utils.serializeCombinatorContract("or zero one"), [3, 0, 1]);
        });

        it('Correctly serializes a then combinator', function() {
            assert.deepEqual(Utils.serializeCombinatorContract("then zero one"), [7, 0, 1]);
        });

        it('Correctly deserializes basic smart contracts', function() {
            var contracts = ["zero", "one"];
            var serialized = contracts.map(elem => Utils.serializeCombinatorContract(elem));

            assert.deepEqual(serialized.map(elem => Utils.deserializeCombinatorContract(0, elem).getContract()), contracts);
        });

        it('Correctly deserializes a give combinator', function() {
            var contract = "give one";
            var serialized = Utils.serializeCombinatorContract(contract);

            assert.equal(Utils.deserializeCombinatorContract(0, serialized).getContract(), contract);
        });

        it('Correctly deserializes a get combinator', function() {
            var contract = "get one";
            var serialized = Utils.serializeCombinatorContract(contract);

            assert.equal(Utils.deserializeCombinatorContract(0, serialized).getContract(), contract);
        });

        it('Correctly deserializes a scale combinator with a scale value', function() {
            var contract = "scale 123456789 one";
            var serialized = Utils.serializeCombinatorContract(contract);

            assert.equal(Utils.deserializeCombinatorContract(0, serialized).getContract(), contract);
        });

        it('Correctly deserializes a scale combinator with an observable', function() {
            var contract = "scale namE013 " + uninvolved.address + " one";
            var serialized = Utils.serializeCombinatorContract(contract);

            assert.equal(Utils.deserializeCombinatorContract(0, serialized).getContract(), "scale namE013 <" + uninvolved.address + "> one");
        });

        it('Correctly deserializes a truncate combinator', function() {
            var contract = "truncate 123456789 one";
            var serialized = Utils.serializeCombinatorContract(contract);

            assert.equal(
                Utils.deserializeCombinatorContract(0, serialized).getContract(),
                "truncate <" + Utils.unixTimestampToDateString(123456789) + "> one"
            );
        });

        it('Correctly deserializes an anytime combinator', function() {
            var contract = "anytime one";
            var serialized = Utils.serializeCombinatorContract(contract);

            assert.equal(Utils.deserializeCombinatorContract(0, serialized).getContract(), contract);
        });

        it('Correctly deserializes an and combinator', function() {
            var contract = "and zero one";
            var serialized = Utils.serializeCombinatorContract(contract);

            assert.equal(Utils.deserializeCombinatorContract(0, serialized).getContract(), contract);
        });

        it('Correctly deserializes an or combinator', function() {
            var contract = "or zero one";
            var serialized = Utils.serializeCombinatorContract(contract);

            assert.equal(Utils.deserializeCombinatorContract(0, serialized).getContract(), contract);
        });

        it('Correctly deserializes a then combinator', function() {
            var contract = "then zero one";
            var serialized = Utils.serializeCombinatorContract(contract);

            assert.equal(Utils.deserializeCombinatorContract(0, serialized).getContract(), contract);
        });
    });

    describe('Contract verification tests', function() {
        it('Verifies a valid contract', function() {
            var res = Utils.verifyContract(
                "and one or zero scale 123456789 scale namE10 " + uninvolved.address + " anytime truncate 123 get give then one zero"
            );

            assert.equal(res.error, undefined);
        });

        it('Does not verify a contract missing combinators', function() {
            var res = Utils.verifyContract("and one");

            assert.notEqual(res.error, undefined);
        });

        it('Does not verify a contract with an invalid combinator', function() {
            var res = Utils.verifyContract("on");

            assert.notEqual(res.error, undefined);
        });

        it('Does not verify a contract with an invalid date', function() {
            var res = Utils.verifyContract("truncate -1 one");

            assert.notEqual(res.error, undefined);
        });

        it('Does not verify a contract with an invalid arbiter address', function() {
            var res = Utils.verifyContract("scale name " + uninvolved.address + "123 one");

            assert.notEqual(res.error, undefined);
        });
    });

    describe('Contract interaction tests', function() {
        describe('Simple contract interactions', function() {
            var contract;
            var contractDefinition = "one";
            var contractValue = 1;

            beforeEach(function() {
                return deploy(contractDefinition, false).then(res => {
                    contract = res;
                });
            });

            it('Correctly gets the holder of the given contract', function() {
                return Utils.getHolder(contract, uninvolved.address).then(function(res) {
                    assert.equal(res, holder.address);
                });
            });
    
            it('Correctly gets the counter-party of the given contract', function() {
                return Utils.getCounterParty(contract, uninvolved.address).then(function(res) {
                    assert.equal(res, counterParty.address);
                });
            });

            it('Correctly gets the balance of the holder from the given contract', function() {
                return Utils.getBalance(contract, holder.address, true).then(function(res) {
                    assert.equal(res, 0);

                    return Utils.acquireContract(contract, holder.address).then(function() {
                        return Utils.getBalance(contract, holder.address, true).then(function(res) {
                            assert.equal(res, contractValue);
                        });
                    });
                });
            });

            it('Correctly gets the balance of the counter-party from the given contract', function() {
                return Utils.getBalance(contract, counterParty.address, false).then(function(res) {
                    assert.equal(res, 0);

                    return Utils.acquireContract(contract, holder.address).then(function() {
                        return Utils.getBalance(contract, counterParty.address, false).then(function(res) {
                            assert.equal(res, -contractValue);
                        });
                    });
                });
            });
    
            it('Correctly gets the combinator contract from the given contract', function() {
                return Utils.getCombinatorContract(contract, holder.address).then(function(res) {
                    assert.equal(res, contractDefinition);
                });
            });  
    
            it('Correctly gets whether the given contract is concluded', function() {
                return Utils.getConcluded(contract, holder.address).then(function(res) {
                    assert.equal(res, false);

                    return Utils.acquireContract(contract, holder.address).then(function() {
                        return Utils.getConcluded(contract, holder.address).then(function(res) {
                            assert.equal(res, true);
                        });
                    });
                });
            });
    
            it('Correctly gets whether the given contract is using gas', function() {
                return Utils.getUseGas(contract, uninvolved.address).then(function(res) {
                    assert.strictEqual(res, false);
                });
            });
    
            it('Correctly gets the last-updated time on the given contract', function() {
                return Utils.getLastUpdated(contract, holder.address).then(function(res) {
                    assert.equal(Utils.dateToUnixTimestamp(new Date()) - res < 2, true);
                });
            });

            it('Correctly acquires the given contract', function() {
                return Utils.acquireContract(contract, holder.address).then(function() {
                    return Utils.getBalance(contract, holder.address, true).then(function(res) {
                        assert.equal(res, contractValue);
                    });
                });
            });

            it('Correctly stakes Ether in the given contract', function() {
                var stake = 100;

                return Utils.stake(contract, holder.address, stake).then(function() {
                    return Utils.getBalance(contract, holder.address, true).then(function(res) {
                        assert.equal(res, stake);
                    });
                });
            });
    
            it('Correctly withdraws Ether from the given contract', function() {
                var stake = 100;
                var withdrawal = 50;

                return Utils.stake(contract, holder.address, stake).then(function() {
                    return Utils.withdraw(contract, holder.address, withdrawal).then(function() {
                        return Utils.getBalance(contract, holder.address, true).then(function(res) {
                            assert.equal(res, stake - withdrawal);
                        });
                    });
                });
            });  
        });

        describe('Or-choice interaction', function() {
            var contract;

            beforeEach(function() {
                return deploy("or or or zero one zero zero").then(function(res) {
                    contract = res;
                });
            });

            it('Correctly gets the or-choices from the given contract', function() {
                return Utils.setOrChoice(contract, holder.address, 0, true).then(function() {
                    return Utils.setOrChoice(contract, holder.address, 1, false).then(function() {
                        return Utils.getOrChoices(contract, holder.address).then(function(res) {
                            var expectedOrChoices = [
                                new Utils.Option(true),
                                new Utils.Option(false),
                                new Utils.Option(undefined)
                            ];

                            assert.deepEqual(res, expectedOrChoices);
                        });
                    });
                });
            });
    
            it('Correctly sets or-choices on the given contract', function() {
                return Utils.setOrChoice(contract, holder.address, 0, true).then(function() {
                    return Utils.setOrChoice(contract, holder.address, 1, true).then(function() {
                        return Utils.setOrChoice(contract, holder.address, 2, false).then(function() {
                            return Utils.acquireContract(contract, holder.address).then(function() {
                                return Utils.getBalance(contract, holder.address, true).then(function(res) {
                                    assert.equal(res, 1);
                                });
                            });
                        });
                    });
                });
            });

            it('Correctly updates the given contract', function() {
                return Utils.acquireContract(contract, holder.address).then(function(res) {
                    return Utils.setOrChoice(contract, holder.address, 0, false).then(function() {
                        return Utils.updateContract(contract, holder.address).then(function() {
                            return Utils.getConcluded(contract, holder.address).then(function(res) {
                                assert.equal(res, true);
                            });
                        });
                    });
                });
            });
        });

        describe('Obs-entry interaction', function() {
            var contract;
            var names = [
                "name0",
                "naMe1"
            ];

            beforeEach(function() {
                return deploy(["scale", names[0], uninvolved.address , "scale", names[1], holder.address, "one"].join(" ")).then(function(res) {
                    contract = res;
                });
            });

            it('Correctly gets the obs-entries from the given contract', function() {
                return Utils.getObsEntries(contract, holder.address).then(function(res) {
                    var expectedObsEntries = [
                        new Utils.ObservableEntry(uninvolved.address, undefined, names[0], 0),
                        new Utils.ObservableEntry(holder.address, undefined, names[1], 1)
                    ];

                    assert.deepEqual(res, expectedObsEntries);
                });
            });

            it('Correctly sets obs-entries on the given contract', function() {
                var obsValues = [
                    10,
                    -25
                ];

                return Utils.setObsValue(contract, uninvolved.address, 0, obsValues[0]).then(function() {
                    return Utils.setObsValue(contract, holder.address, 1, obsValues[1]).then(function() {
                        return Utils.getObsEntries(contract, holder.address).then(function(res) {
                            var expectedObsEntries = [
                                new Utils.ObservableEntry(uninvolved.address, obsValues[0], names[0], 0),
                                new Utils.ObservableEntry(holder.address, obsValues[1], names[1], 1)
                            ];

                            assert.deepEqual(res, expectedObsEntries);
                        });
                    });
                });
            });
        });

        describe('Acquisition time interaction', function() {
            var contract;

            beforeEach(function() {
                return deploy("anytime anytime one").then(function(res) {
                    contract = res;
                });
            });

            it('Correctly gets the acquisition-times from the given contract', function() {
                return Utils.getAcquisitionTimes(contract, holder.address).then(function(res) {
                    var expectedAcquisitionTimes = [
                        new Utils.Option(undefined),
                        new Utils.Option(undefined),
                        new Utils.Option(undefined)
                    ];

                    assert.deepEqual(res, expectedAcquisitionTimes);
                });
            });

            it('Correctly acquires the given contract\'s sub-contract', function() {
                return Utils.acquireContract(contract, holder.address).then(function() {
                    return Utils.acquireSubContract(contract, holder.address, 0).then(function() {
                        return Utils.acquireSubContract(contract, holder.address, 1).then(function() {
                            return Utils.getAcquisitionTimes(contract, holder.address).then(function(res) {
                                var unixTime = Utils.dateToUnixTimestamp(new Date());
                                var dateDifferences = res.map(elem => unixTime - elem.getValue());

                                assert.equal(res.every(elem => elem.isDefined()), true);
                                assert.equal(dateDifferences.every(elem => elem < 2), true);
                            });
                        });
                    });
                });
            });
        });
    });
});