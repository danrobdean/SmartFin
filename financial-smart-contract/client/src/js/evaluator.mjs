import { dateToUnixTimestamp } from "./contract-utils.mjs";

/**
 * Class which handles memoised evaluation of financial contracts.
 */
export default class Evaluator {
    /**
     * The financial contract string.
     */
    contract;

    /**
     * Array of the financial contract's atoms.
     */
    combinators;

    /**
     * The horizon of the contract.
     */
    horizon;

    /**
     * The relevant time slices of the top-level contract.
     */
    timeSlices;

    /**
     * The relevant time slices of the anytime combinators in the contract.
     */
    anytimeTimeSlices;

    /**
     * Sets the financial contract which is to be evaluated.
     * @param contract The financial contract string.
     */
    setContract(contract) {
        if (this.contract !== contract) {
            this.contract = contract.toLowerCase();

            // Set the array of combinator atoms.
            this.combinators = this.contract.split(/[ \(\),]/).filter(elem => elem !== "");

            var result = this._processCombinators(0);
            this.horizon = result.horizon;
            this.timeSlices = result.timeSlices;
            this.anytimeTimeSlices = result.anytimeTimeSlices;
        }
    }

    /**
     * Gets the contract.
     */
    getContract() {
        return this.contract;
    }

    /**
     * Gets the horizon of the contract.
     */
    getHorizon() {
        return this.horizon;
    }

    /**
     * Gets the set of relevant time slices for the contract.
     */
    getTimeSlices() {
        return this.timeSlices.clone();
    }

    /**
     * Gets the array of anytime time slice sets for the contract, in order of occurrence.
     */
    getAnytimeTimeSlices() {
        return this.anytimeTimeSlices;
    }

    /**
     * Gets the horizon of the given financial contract, or undefined if none exists.
     * @param i The index to start processing the associated combinator contract at.
     */
    _processCombinators(i) {
        switch (this.combinators[i]) {
            case "zero":
            case "one":
                return new ProcessResult(undefined, i + 1, new TimeSlices(), []);

            case "truncate":
                var horizon;
                var subHorizonRes;

                if (this.combinators[i + 1].indexOf("<") != -1) {
                    // Time is pretty date string, will be split up into 2 elements
                    horizon = this._dateOrUnixToHorizon(this.combinators.slice(i + 1, i + 3).join(", "));
                    subHorizonRes = this._processCombinators(i + 3);
                } else {
                    // Time is unix
                    horizon = this._dateOrUnixToHorizon(this.combinators[i + 1]);
                    subHorizonRes = this._processCombinators(i + 2);
                }

                // Adjust TimeSlices
                var timeSlices = subHorizonRes.timeSlices;
                var finalHorizon = this._getMinHorizon(horizon, subHorizonRes.horizon);
                timeSlices.cutTail(finalHorizon);

                return new ProcessResult(finalHorizon, subHorizonRes.tailIndex, timeSlices, subHorizonRes.anytimeTimeSlices);

            case "give":
                return this._processCombinators(i + 1);

            case "get":
                var subHorizonRes = this._processCombinators(i + 1);

                // Cut off time slice before horizon, as acquiring get c before H(c)
                // will acquire c at H(c), so the time c is acquired makes no difference.
                subHorizonRes.timeSlices.cutHead(subHorizonRes.horizon);

                return subHorizonRes;

            case "anytime":
                var subHorizonRes = this._processCombinators(i + 1);

                // Add current contract's time slices to front of anytime time slices array.
                subHorizonRes.anytimeTimeSlices.unshift(subHorizonRes.timeSlices.clone());

                return subHorizonRes;

            case "scale":
                if (this.combinators[i + 1] == "obs") {
                    return this._processCombinators(i + 3);
                } else {
                    return this._processCombinators(i + 2);
                }

            case "and":
            case "or":
                var subHorizonRes0 = this._processCombinators(i + 1);
                var subHorizonRes1 = this._processCombinators(subHorizonRes0.tailIndex);
                var finalHorizon = this._getMaxHorizon(subHorizonRes0.horizon, subHorizonRes1.horizon);

                // Merge time slices
                var timeSlices = subHorizonRes0.timeSlices;
                timeSlices.merge(subHorizonRes1.timeSlices);

                var anytimeTimeSlices = subHorizonRes0.anytimeTimeSlices.concat(subHorizonRes1.anytimeTimeSlices);
                
                return new ProcessResult(finalHorizon, subHorizonRes1.tailIndex, timeSlices, anytimeTimeSlices);

            case "then":
                var subHorizonRes0 = this._processCombinators(i + 1);
                var subHorizonRes1 = this._processCombinators(subHorizonRes0.tailIndex);
                var finalHorizon = this._getMaxHorizon(subHorizonRes0.horizon, subHorizonRes1.horizon);

                // Merge time slices, only adding sub-combinator 2's slices after sub-combinator 1's horizon
                var timeSlices = subHorizonRes0.timeSlices;
                timeSlices.mergeAfter(subHorizonRes1.timeSlices);

                var anytimeTimeSlices = subHorizonRes0.anytimeTimeSlices.concat(subHorizonRes1.anytimeTimeSlices);
                
                return new ProcessResult(finalHorizon, subHorizonRes1.tailIndex, timeSlices, anytimeTimeSlices);
        }
    }

    /**
     * Returns the maximum of the two given horizons.
     * @param h0 The first horizon.
     * @param h1 The second horizon.
     */
    _getMaxHorizon(h0, h1) {
        if (h0 === undefined || h1 === undefined) {
            return undefined;
        } else {
            return (h0 < h1) ? h1 : h0;
        }
    }

    /**
     * Returns the minimum of the two given horizons.
     * @param h0 The first horizon.
     * @param h1 The second horizon.
     */
    _getMinHorizon(h0, h1) {
        if (h0 === undefined) {
            return h1;
        } else if (h1 === undefined) {
            return h0;
        } else {
            return (h0 > h1) ? h1 : h0;
        }
    }

    /**
     * Converts a pretty-printed date or a unix timestamp to a horizon number.
     * @param date The pretty-printed date or unix timestamp.
     */
    _dateOrUnixToHorizon(date) {
        // If date is a pretty string or unix timestring, process it into a unix timestamp
        if (typeof date == "string") {
            // If date is angle-bracketed, remove them
            if (date.indexOf("<") != -1) {
                date = dateToUnixTimestamp(new Date(date.slice(1, -1)));
            }

            // Return date as int, not string
            date = parseInt(date);
        }

        // If date is `undefined`, or a number, just return it
        
        return date;
    }
}

/**
 * The result of processing a set of combinators, contains the horizon, the
 * tail following the final combinator examined, and the set of time slices.
 */
class ProcessResult {
    /**
     * The contract horizon.
     */
    horizon;

    /**
     * The index of the tail of combinators following the last processed combinator.
     */
    tailIndex;

    /**
     * The TimeSlices of the contract.
     */
    timeSlices;

    /**
     * The array of TimeSlices of the anytime combinators in the contract.
     */
    anytimeTimeSlices;

    /**
     * Initialises a new instance of this class.
     * @param horizon The horizon of the getHorizon call.
     * @param tailIndex The index of the tail of the set of combinators after the last combinator of the getHorizon call.
     * @param timeSlices The TimeSlices of the contract.
     * @param anytimeTimeSlices The array of TimeSlices of the anytime combinators in the contract.
     */
    constructor(horizon, tailIndex, timeSlices, anytimeTimeSlices) {
        this.horizon = horizon;
        this.tailIndex = tailIndex;
        this.timeSlices = timeSlices;
        this.anytimeTimeSlices = anytimeTimeSlices;
    }
}

/**
 * Represents a set of time slices, with several operations.
 */
export class TimeSlices {
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