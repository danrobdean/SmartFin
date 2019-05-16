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
     * Sets the financial contract which is to be evaluated.
     * @param contract The financial contract string.
     */
    setContract(contract) {
        this.contract = contract.toLowerCase();
        this.combinators = this.contract.split(/[ \(\),]/).filter(elem => elem !== "");
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
        return this._getHorizon(this.combinators).horizon;
    }

    /**
     * Gets the horizon of the given financial contract, or undefined if none exists.
     * @param combinators Array of the financial contract atoms to get the horizon of.
     */
    _getHorizon(combinators) {
        try {
            switch (combinators[0]) {
                case "zero":
                case "one":
                    return new HorizonResult(undefined, combinators.slice(1));
    
                case "truncate":
                    var horizon;
                    var subHorizonRes;

                    if (combinators[1].indexOf("<") != -1) {
                        horizon = this._dateOrUnixToHorizon(combinators.slice(1, 3).join(", "));
                        subHorizonRes = this._getHorizon(combinators.slice(3));
                    } else {
                        horizon = this._dateOrUnixToHorizon(combinators[1]);
                        subHorizonRes = this._getHorizon(combinators.slice(2));
                    }

                    return new HorizonResult(this._getMinHorizon(horizon, subHorizonRes.horizon), subHorizonRes.tail);
    
                case "give":
                case "get":
                case "anytime":
                    return this._getHorizon(combinators.slice(1));
    
                case "scale":
                    if (combinators[1] == "obs") {
                        return this._getHorizon(combinators.slice(3));
                    } else {
                        return this._getHorizon(combinators.slice(2));
                    }
    
                case "and":
                case "or":
                case "then":
                    var subHorizonRes0 = this._getHorizon(combinators.slice(1));
                    var subHorizonRes1 = this._getHorizon(subHorizonRes0.tail);
                    
                    return new HorizonResult(this._getMaxHorizon(subHorizonRes0.horizon, subHorizonRes1.horizon), subHorizonRes1.tail);
            }
        } catch (err) {
            throw "Cannot get horizon, contract is invalid";
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
 * The result of a call to getHorizon, contains the horizon and the tail following the final combinator examined.
 */
class HorizonResult {
    /**
     * The contract horizon.
     */
    horizon;

    /**
     * The tail array of combinators.
     */
    tail;

    /**
     * Initialises a new instance of this class.
     * @param horizon The horizon of the getHorizon call.
     * @param tail The tail of the set of combinators after the last combinator of the getHorizon call.
     */
    constructor(horizon, tail) {
        this.horizon = horizon;
        this.tail = tail;
    }
}