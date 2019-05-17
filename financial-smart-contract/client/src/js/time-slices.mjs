/**
 * Represents a set of time slices, with several operations.
 */
export default class TimeSlices {
    /**
     * The set of time slices. Must remain sorted. Each time slice lasts up to and including the slice's value.
     */
    _slices = [];

    /**
     * Gets the set of time slices.
     */
    getSlices() {
        return this._slices.slice();
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

        var index = this._slices.findIndex(elem => elem >= time);
        if (index != -1) {
            this._slices = this._slices.slice(0, index);
        }

        this.split(time);
    }

    /**
     * Cuts off the time slice set at the given time, removing all preceding slices, and splitting at the given time.
     * @param time 
     */
    cutHead(time) {
        // Undefined time is equivalent to an "infinite" horizon, so everything should be removed.
        // For example, get truncate infinity c can only be acquired at infinity, so it should only have one time slice.
        if (time === undefined) {
            this._slices = [];
            return;
        }

        var index = this._slices.findIndex(elem => elem >= time);
        if (index != -1) {
            this._slices.splice(0, index);
        } else {
            this._slices = [];
        }

        this.split(time);
    }

    /**
     * Splits a time slice at the given time.
     * @param time The time to split a slice at.
     */
    split(time) {
        if (!this._slices.includes(time)) {
            this._slices.push(time);
            this._slices.sort();
        }
    }

    /**
     * Merges another TimeSlices object with this one.
     * @param other The TimeSlices object to merge.
     */
    merge(other) {
        for (var time of other.getSlices()) {
            this.split(time);
        }
    }

    /**
     * Merges another TimeSlices object with this one, without splitting any existing slices.
     * @param other The TimeSlices object to merge.
     */
    mergeAfter(other) {
        var otherSlices = other.getSlices();

        var startIndex;
        if (this._slices.length > 0) {
            startIndex = otherSlices.findIndex(elem => elem > this._slices[this._slices.length - 1])

            if (startIndex == -1) {
                return;
            }
        } else {
            startIndex = 0;
        }

        for (var i = startIndex; i < otherSlices.length; i++) {
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
}