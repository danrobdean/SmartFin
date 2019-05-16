import assert from "assert";

import { holder, uninvolved } from "./common";
import { unlockAccount, serializeAddress, deserializeAddress } from "../src/js/contract-utils.mjs";


describe('Contract utility tests', function() {
    it('Unlocks accounts without error', function() {
        return unlockAccount(holder.address, holder.password);
    });

    it('Correctly serializates/deserializes address', function() {
        var address = uninvolved.address;
        var serialized = serializeAddress(address);
        var deserialized = deserializeAddress(serialized);
        assert.equal(deserialized, address);
    });
});