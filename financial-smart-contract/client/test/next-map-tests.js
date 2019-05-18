import assert from "assert";

import NextMap from "../src/js/next-map.mjs";

describe('Next Map tests', function() {
    var nextMap;

    beforeEach(function() {
        nextMap = new NextMap();
    });

    it('Returns a value when its key is searched for', function() {
        nextMap.add(0, "0");
        nextMap.add(1, "1");
        nextMap.add(2, "2");

        assert.equal(nextMap.getNextValue(1), "1");
    });

    it('Returns the value of the next largest key when a non-existing key is searched for', function() {
        nextMap.add(0, "0");
        nextMap.add(2, "2");

        assert.equal(nextMap.getNextValue(1), "2");
    });
});