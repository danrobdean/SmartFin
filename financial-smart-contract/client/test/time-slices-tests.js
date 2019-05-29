import assert from "assert";

import TimeSlices from "./../src/js/time-slices.mjs";
import TimeSlice from "./../src/js/time-slice.mjs";

describe('TimeSlices tests', function() {
    var timeSlices;

    beforeEach(function() {
        timeSlices = new TimeSlices();
    });

    it('Has one complete slice initially', function() {
        assert.deepEqual(timeSlices.getSlices(), [new TimeSlice(0, undefined)]);
    });

    it('Splits time slices correctly', function() {
        timeSlices.split(1);
        timeSlices.split(3);
        timeSlices.split(2);

        assert.deepEqual(timeSlices.getSlices(), [new TimeSlice(0, 1), new TimeSlice(2, 2), new TimeSlice(3, 3), new TimeSlice(4, undefined)]);
    });

    it('Cuts off time slices correctly', function() {
        timeSlices.split(1);
        timeSlices.split(3);
        timeSlices.cutTail(2);

        assert.deepEqual(timeSlices.getSlices(), [new TimeSlice(0, 1), new TimeSlice(2, 2)]);
    });

    it('Concatenates initial time slices correctly', function() {
        timeSlices.split(1);
        timeSlices.split(2);
        timeSlices.split(3);
        timeSlices.split(4);
        timeSlices.concatHead(3);

        assert.deepEqual(timeSlices.getSlices(), [new TimeSlice(0, 3), new TimeSlice(4, 4), new TimeSlice(5, undefined)]);
    });

    it('Gets valid time slices correctly', function() {
        timeSlices.split(1);
        timeSlices.split(2);
        timeSlices.split(3);
        timeSlices.split(4);

        assert.deepEqual(timeSlices.getValidSlices(3), [new TimeSlice(3, 3), new TimeSlice(4, 4), new TimeSlice(5, undefined)]);
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

        assert.deepEqual(timeSlices.getSlices(), [new TimeSlice(0, 1), new TimeSlice(2, 2), new TimeSlice(3, 3), new TimeSlice(4, 4), new TimeSlice(5, 5), new TimeSlice(6, 6), new TimeSlice(7, undefined)]);
    });

    it('Merge after merges time slices correctly', function() {
        var otherTimeSlices = new TimeSlices();

        timeSlices.cutTail(1);
        timeSlices.split(3);
        timeSlices.split(5);

        otherTimeSlices.split(2);
        otherTimeSlices.split(4);
        otherTimeSlices.split(6);
        otherTimeSlices.split(7);

        timeSlices.mergeAfter(otherTimeSlices);

        assert.deepEqual(timeSlices.getSlices(), [new TimeSlice(0, 1), new TimeSlice(2, 3), new TimeSlice(4, 5), new TimeSlice(6, 6), new TimeSlice(7, 7), new TimeSlice(8, undefined)]);
    });
});