import assert from "assert";

import TimeSlices from "./../src/js/time-slices.mjs";
import TimeRange from "./../src/js/time-range.mjs";

describe('TimeSlices tests', function() {
    var timeSlices;

    beforeEach(function() {
        timeSlices = new TimeSlices();
    });

    it('Has one complete slice initially', function() {
        assert.deepEqual(timeSlices.getSlices(), [new TimeRange(0, undefined)]);
    });

    it('Splits time slices correctly', function() {
        timeSlices.split(1);
        timeSlices.split(3);
        timeSlices.split(2);

        assert.deepEqual(timeSlices.getSlices(), [new TimeRange(0, 1), new TimeRange(2, 2), new TimeRange(3, 3), new TimeRange(4, undefined)]);
    });

    it('Cuts off time slices correctly', function() {
        timeSlices.split(1);
        timeSlices.split(3);
        timeSlices.cutTail(2);

        assert.deepEqual(timeSlices.getSlices(), [new TimeRange(0, 1), new TimeRange(2, 2)]);
    });

    it('Concatenates initial time slices correctly', function() {
        timeSlices.split(1);
        timeSlices.split(2);
        timeSlices.split(3);
        timeSlices.split(4);
        timeSlices.concatHead(3);

        assert.deepEqual(timeSlices.getSlices(), [new TimeRange(0, 3), new TimeRange(4, 4), new TimeRange(5, undefined)]);
    });

    it('Gets valid time slices correctly', function() {
        timeSlices.split(1);
        timeSlices.split(2);
        timeSlices.split(3);
        timeSlices.split(4);

        assert.deepEqual(timeSlices.getValidSlices(3), [new TimeRange(3, 3), new TimeRange(4, 4), new TimeRange(5, undefined)]);
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

        assert.deepEqual(timeSlices.getSlices(), [new TimeRange(0, 1), new TimeRange(2, 2), new TimeRange(3, 3), new TimeRange(4, 4), new TimeRange(5, 5), new TimeRange(6, 6), new TimeRange(7, undefined)]);
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

        assert.deepEqual(timeSlices.getSlices(), [new TimeRange(0, 1), new TimeRange(2, 3), new TimeRange(4, 5), new TimeRange(6, 6), new TimeRange(7, 7), new TimeRange(8, undefined)]);
    });
});