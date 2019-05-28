import TimeRange from "./time-range.mjs";

/**
 * Represents a set of time slices, with several operations.
 */
export default class TimeSlices {
    /**
     * The set of time slices. Must remain sorted, distinct, and start at 0.
     */
    _slices = [new TimeRange(0, undefined)];

    /**
     * Gets the set of time slices.
     */
    getSlices() {
        return this._slices.slice();
    }

    /**
     * Gets the end time of the time slices set.
     */
    getEndTime() {
        return this._slices[this._slices.length - 1].getEnd();
    }

    /**
     * Splits a time slice at the given time.
     * @param time The time to split a slice at.
     */
    split(time) {
        var index = this._slices.findIndex(elem => elem.includes(time));
        var oldRange;
        if (index == -1) {
            var endTime = this.getEndTime();
            this._slices.push(new TimeRange(endTime + 1, time));
        } else {
            if (time === undefined) {
                // undefined = non-finite time, cannot be split
                return;
            }

            var oldRange = this._slices[index];

            if (oldRange.getEnd() == time) {
                // Already split at this time, so nothing more to do.
                return;
            }
            var range0 = new TimeRange(oldRange.getStart(), time);
            var range1 = new TimeRange(time + 1, oldRange.getEnd());
    
            this._slices.splice(index, 1);
            this._slices.push(range0, range1);
            this._slices.sort(TimeRange.compareRangeEnds);
        }
    }

    /**
     * Cuts off the time slice set at the given time, removing all following slices, and splitting at the given time.
     * @param time The time to cut off the time slices after.
     */
    cutTail(time) {
        // Undefined time is equivalent to an "infinity" horizon, so nothing can be cut off.
        // For example, truncate infinity c can be acquired at any time, so it is equal to just c, so no change should be made.
        if (time === undefined) {
            return;
        }

        this.split(time);
        var index = this._slices.findIndex(elem => elem.includes(time));
        this._slices = this._slices.slice(0, index + 1);
    }

    /**
     * Concatenates the time slice set up to the given time, combining all preceding slices, and splitting at the given time.
     * @param time 
     */
    concatHead(time) {
        // Undefined time is equivalent to an "infinite" horizon, so everything should be removed.
        // For example, get one can only be acquired at infinity, so it should only have one time slice.
        if (time === undefined) {
            this._slices = [];
        } else {
            // Split and remove preceding slices
            this.split(time);
    
            var index = this._slices.findIndex(elem => elem.includes(time));
            this._slices.splice(0, index + 1);
        }

        // Add new concatenated slice
        this._slices.unshift(new TimeRange(0, time));
    }

    /**
     * Merges another TimeSlices object with this one.
     * @param other The TimeSlices object to merge.
     */
    merge(other) {
        for (var time of other.getSlices()) {
            this.split(time.getEnd());
        }
    }

    /**
     * Merges another TimeSlices object with this one, without splitting any existing slices.
     * @param other The TimeSlices object to merge.
     */
    mergeAfter(other) {
        var endTime = this.getEndTime();
        if (other.getEndTime() < endTime) {
            // Our time slices end after theirs, nothing to merge
            return;
        }

        // Split other slices at our end time
        var otherClone = other.clone();
        otherClone.split(endTime);
        var otherSlices = otherClone.getSlices();

        // Delete every slice up to our end time in otherSlices
        var startIndex;
        startIndex = otherSlices.findIndex(elem => elem.includes(endTime));

        if (startIndex == -1) {
            // Time slices must start from 0, so their last time slice is before our last time slice
            return;
        }

        for (var i = startIndex + 1; i < otherSlices.length; i++) {
            this._slices.push(otherSlices[i]);
        }
    }

    /**
     * Clones this object.
     */
    clone() {
        var result = new TimeSlices();
        result._slices = this.getSlices();
        return result;
    }

    /**
     * Gets the list of time slices that are valid at the given time (i.e. remove slices before this time).
     * @param currentTime The current time.
     */
    getValidSlices(currentTime) {
        var clone = this.clone();

        // Split just before current time
        clone.split(currentTime - 1);

        // Remove ranges before current time
        var nowIndex = clone._slices.findIndex(elem => elem.includes(currentTime));
        return clone._slices.slice(nowIndex);
    }
}