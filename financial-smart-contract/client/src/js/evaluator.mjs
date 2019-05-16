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
     * The relevant time slices of the contract.
     */
    timeSlices;

    /**
     * Sets the financial contract which is to be evaluated.
     * @param contract The financial contract string.
     */
    setContract(contract) {
        if (this.contract !== contract) {
            this.contract = contract.toLowerCase();

            // Set the array of combinator atoms.
            this.combinators = this.contract.split(/[ \(\),]/).filter(elem => elem !== "");

            var result = this._processCombinators(this.combinators);
            this.horizon = result.horizon;
            this.timeSlices = result.timeSlices;
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
        return this.timeSlices;
    }

    /**
     * Gets the horizon of the given financial contract, or undefined if none exists.
     * @param combinators Array of the financial contract atoms to get the horizon of.
     */
    _processCombinators(combinators) {
        switch (combinators[0]) {
            case "zero":
            case "one":
                return new ProcessResult(undefined, combinators.slice(1), new TimeSlices());

            case "truncate":
                var horizon;
                var subHorizonRes;

                if (combinators[1].indexOf("<") != -1) {
                    // Time is pretty date string, will be split up into 2 elements
                    horizon = this._dateOrUnixToHorizon(combinators.slice(1, 3).join(", "));
                    subHorizonRes = this._processCombinators(combinators.slice(3));
                } else {
                    // Time is unix
                    horizon = this._dateOrUnixToHorizon(combinators[1]);
                    subHorizonRes = this._processCombinators(combinators.slice(2));
                }

                // Adjust TimeSlices
                var timeSlices = subHorizonRes.timeSlices;
                var finalHorizon = this._getMinHorizon(horizon, subHorizonRes.horizon);
                timeSlices.cut(finalHorizon);

                return new ProcessResult(finalHorizon, subHorizonRes.tail, timeSlices);

            case "give":
            case "get":
            case "anytime":
                return this._processCombinators(combinators.slice(1));

            case "scale":
                if (combinators[1] == "obs") {
                    return this._processCombinators(combinators.slice(3));
                } else {
                    return this._processCombinators(combinators.slice(2));
                }

            case "and":
            case "or":
                var subHorizonRes0 = this._processCombinators(combinators.slice(1));
                var subHorizonRes1 = this._processCombinators(subHorizonRes0.tail);
                var finalHorizon = this._getMaxHorizon(subHorizonRes0.horizon, subHorizonRes1.horizon);

                // Merge time slices
                var timeSlices = subHorizonRes0.timeSlices;
                timeSlices.merge(subHorizonRes1.timeSlices);
                
                return new ProcessResult(finalHorizon, subHorizonRes1.tail, timeSlices);

            case "then":
                var subHorizonRes0 = this._processCombinators(combinators.slice(1));
                var subHorizonRes1 = this._processCombinators(subHorizonRes0.tail);
                var finalHorizon = this._getMaxHorizon(subHorizonRes0.horizon, subHorizonRes1.horizon);

                // Merge time slices, only adding sub-combinator 2's slices after sub-combinator 1's horizon
                var timeSlices = subHorizonRes0.timeSlices;
                timeSlices.mergeAfter(subHorizonRes1.timeSlices);
                
                return new ProcessResult(finalHorizon, subHorizonRes1.tail, timeSlices);
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
     * The tail array of combinators.
     */
    tail;

    /**
     * The TimeSlices of the contract.
     */
    timeSlices;

    /**
     * Initialises a new instance of this class.
     * @param horizon The horizon of the getHorizon call.
     * @param tail The tail of the set of combinators after the last combinator of the getHorizon call.
     * @param timeSlices The TimeSlices of the contract.
     */
    constructor(horizon, tail, timeSlices) {
        this.horizon = horizon;
        this.tail = tail;
        this.timeSlices = timeSlices;
    }
}

/**
 * Represents a set of time slices, with several operations.
 */
export class TimeSlices {
    /**
     * The set of time slices. Must remain sorted.
     */
    _slices = [];

    /**
     * Gets the set of time slices.
     */
    getSlices() {
        return this._slices;
    }

    /**
     * Cuts off the time slice set at the given time, removing all following slices. If no slice follows the given time, then do nothing.
     * @param time The time to cut off the time slice at.
     */
    cut(time) {
        var index = this._slices.findIndex(elem => elem >= time);
        if (index == -1) {
            return;
        }

        this._slices = this._slices.slice(0, index);
        this._slices.push(time);
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
}