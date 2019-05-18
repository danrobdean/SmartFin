import assert from "assert";

import TimeSlices from "./../src/js/time-slices.mjs";

describe('TimeSlices tests', function() {
    var timeSlices;

    beforeEach(function() {
        timeSlices = new TimeSlices();
    });

    it('Has no slices initially', function() {
        assert.deepEqual(timeSlices.getSlices(), []);
    });

    it('Splits time slices correctly', function() {
        timeSlices.split(1);
        timeSlices.split(3);
        timeSlices.split(2);

        assert.deepEqual(timeSlices.getSlices(), [1, 2, 3]);
    });

    it('Cuts off time slices correctly', function() {
        timeSlices.split(1);
        timeSlices.split(3);
        timeSlices.cutTail(2);

        assert.deepEqual(timeSlices.getSlices(), [1, 2]);
    });

    it('Merges time slices correctly', function() {
        var otherTimeSlices = new TimeSlices();

        timeSlices.split(1);
        timeSlices.split(3);
        timeSlices.split(5);

        otherTimeSlices.split(2);
        otherTimeSlices.split(4);
        otherTimeSlices.split(6);

        timeSlices.merge(otherTimeSlices);

        assert.deepEqual(timeSlices.getSlices(), [1, 2, 3, 4, 5, 6]);
    });

    it('Merge after merges time slices correctly', function() {
        var otherTimeSlices = new TimeSlices();

        timeSlices.split(1);
        timeSlices.split(3);
        timeSlices.split(5);

        otherTimeSlices.split(2);
        otherTimeSlices.split(4);
        otherTimeSlices.split(6);
        otherTimeSlices.split(7);

        timeSlices.mergeAfter(otherTimeSlices);

        assert.deepEqual(timeSlices.getSlices(), [1, 3, 5, 6, 7]);
    });
});