import { dateToUnixTimestamp } from "./contract-utils.mjs";
import StepThroughOptions from "./step-through-options.mjs";
import TimeSlices from "./time-slices.mjs";

/**
 * Class which handles memoised evaluation of financial contracts.
 */
export default class Evaluator {
    /**
     * The set of combinators which rely upon user input.
     */
    static INPUT_RELIANT_COMBINATORS = ["anytime", "or"];

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
     * The map of combinator indexes to their horizons.
     */
    combinatorHorizonMap;

    /**
     * The map of the combinator indexes to their contracts' tail indexes.
     */
    combinatorTailIndexMap;

    /**
     * The map of combinator indexes to their observable index (if applicable).
     */
    combinatorObsIndexMap;

    /**
     * The relevant time slices of the top-level contract.
     */
    timeSlices;

    /**
     * The relevant time slices of the anytime combinators in the contract.
     */
    anytimeTimeSlices;

    /**
     * The current index of stepping through input-reliant combinators in the contract.
     */
    stepThroughIndex;

    /**
     * The current combinator index of stepping through combinators in the contract.
     */
    stepThroughCombinatorIndex;

    /**
     * The current anytime index from stepping through the combinators in the contract.
     */
    stepThroughAcquisitionTimeIndex;

    /**
     * The current set of options from stepping through the combinator contract.
     */
    stepThroughOptions;

    /**
     * Whether or not the step-through contract evaluation can continue.
     */
    hasNextStep;

    /**
     * The evaluation of the contract with the stepped-through options.
     */
    stepThroughValue;

    /**
     * Sets the financial contract which is to be evaluated.
     * @param contract The financial contract string.
     */
    setContract(contract) {
        if (this.contract !== contract) {
            this._resetState();
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
     * Starts step-through evaluation of the financial contract.
     */
    startStepThroughEvaluation() {
        this._resetState();

        return new StepThroughOptions(StepThroughOptions.TYPE_ACQUISITION_TIME, this.timeSlices.getSlices(), -1);
    }

    /**
     * If the step-through of the contract is concluded, evaluate the value based on the options provided.
     */
    getStepThroughEvaluationValue() {
        if (this.stepThroughValue !== undefined) {
            return this.stepThroughValue;
        }

        var options = this.stepThroughOptions.slice();
        var time = options.shift().value;
        var res = this._stepThroughEvaluate(0, time, options);

        var value = "";

        if (res.obsIndexes.length > 0) {
            value += "obs_" + res.obsIndexes[0].toString() + " * ";
            for (var i = 1; i < res.obsIndexes.length; i++) {
                value += "obs_" + res.obsIndexes[i].toString() + " * "; 
            }
            value 
        }

        value += res.concreteValue.toString() + " Wei";
    }

    /**
     * Returns true if more step-through options can be set, false if otherwise.
     */
    hasNextStep() {
        return this.hasNextStep;
    }

    /**
     * Sets the current step-through evaluation option.
     * @param option The input option for the current input-reliant combinator.
     */
    setStepThroughOption(option) {
        // Set the step through option and increment the combinator-index to the start of the next contract
        if (this.stepThroughIndex == 0) {
            // Set the top-level acquisition time
            this.stepThroughOptions.push(new StepThroughValue(StepThroughValue.TYPE_ACQUISITION_TIME, option, this.stepThroughCombinatorIndex));

            this.stepThroughAcquisitionTimeIndex += 1;
        } else {
            switch (this.combinators[this.stepThroughCombinatorIndex]) {
                case "or":
                    this.stepThroughOptions.push(new StepThroughValue(StepThroughValue.TYPE_OR_CHOICE, option, this.stepThroughCombinatorIndex));

                    this.stepThroughCombinatorIndex += 1;

                    // Skip first sub-combinator if option is false, i.e. second child is chosen
                    if (!option) {
                        this.stepThroughCombinatorIndex = this._getEndOfContract(this.stepThroughCombinatorIndex) + 1;
                    }
                    break;

                case "anytime":
                    this.stepThroughOptions.push(new StepThroughValue(StepThroughValue.TYPE_ACQUISITION_TIME, option, this.stepThroughCombinatorIndex));

                    // Move on to sub-combinator.
                    this.stepThroughCombinatorIndex += 1;
                    this.stepThroughAcquisitionTimeIndex += 1;
                    break;

                default:
                    throw "Tried to set a non-existent step-through option."
            }
        }

        // Increment step-through index, and step-through combinator-index to next input-reliant combinator
        this.stepThroughIndex += 1;

        while (!Evaluator.INPUT_RELIANT_COMBINATORS.includes(this.combinators[this.stepThroughCombinatorIndex])) {
            stepThroughCombinatorIndex += 1;

            if (stepThroughCombinatorIndex >= this.combinators.length) {
                this.hasNextStep = false;
                return !this.hasNextStep;
            }
        }

        // If the next input-reliant combinator is an anytime combinator, check whether the horizon has passed
        if (this.combinators[stepThroughCombinatorIndex] == "anytime") {
            var timeSlices = this.anytimeTimeSlices[this.stepThroughAcquisitionTimeIndex - 1].getSlices();

            var lastAcquireTimeOption = this.stepThroughOptions.slice().reverse().findIndex(option => {
                return option.type == StepThroughValue.TYPE_ACQUISITION_TIME;
            });

            if (timeSlices.length != 0 && timeSlices[timeSlices.length - 1] < lastAcquireTimeOption.value) {
                this.hasNextStep = false;
                this.stepThroughValue = 0;
            }
        }

        return !this.hasNextStep;
    }

    /**
     * Resets the step-through option at the given index, deleting following options.
     * @param option The new step-through option.
     * @param index The index of the step-through option to reset.
     */
    resetStepThroughOption(option, index) {
        if (this.stepThroughIndex <= index) {
            throw "Tried to reset a step-through option which has not yet been set."
        }

        this.stepThroughIndex = index;
        this.stepThroughCombinatorIndex = this.stepThroughOptions[index].combinatorIndex;
        this.stepThroughOptions = this.stepThroughOptions.slice(0, index);
        this.stepThroughAcquisitionTimeIndex = this.stepThroughOptions.reduce((prev, cur) => {
            return (cur.type == StepThroughValue.TYPE_ACQUISITION_TIME) ? prev + 1 : prev;
        }, 0);
        this.hasNextStep = true;

        this.setStepThroughOption(option);
    }

    /**
     * Gets the options for the next step of evaluating the financial contract.
     */
    getNextStepThroughOptions() {
        if (this.stepThroughIndex == 0) {
            return StepThroughOptions(StepThroughOptions.TYPE_ACQUISITION_TIME, this.timeSlices.getSlices(), this.stepThroughCombinatorIndex);
        }

        switch (this.combinators[this.stepThroughCombinatorIndex]) {
            case "or":
                return StepThroughOptions(StepThroughOptions.TYPE_OR_CHOICE, [true, false], this.stepThroughCombinatorIndex);

            case "anytime":
                // Cut off the time-slice options by the current sub-contract's acquisition time
                var lastAcquireTimeOption = this.stepThroughOptions.slice().reverse().findIndex(option => {
                    return option.type == StepThroughValue.TYPE_ACQUISITION_TIME;
                });
                var timeSlices = this.anytimeTimeSlices[this.stepThroughAcquisitionTimeIndex - 1].getSlices();
                var index = timeSlices.findIndex(elem => elem >= lastAcquireTimeOption.value);
                timeSlices = timeSlices.slice(index);

                return StepThroughOptions(StepThroughOptions.TYPE_ACQUISITION_TIME, timeSlices, this.stepThroughCombinatorIndex);

            default:
                throw "Unknown input-reliant combinator found.";
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
                this.combinatorHorizonMap.add(i, finalHorizon);

                return new ProcessResult(finalHorizon, subHorizonRes.tailIndex, timeSlices, subHorizonRes.anytimeTimeSlices);

            case "give":
                var subCombinatorRes = this._processCombinators(i + 1);

                return subCombinatorRes;

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
                var subCombinatorRes;

                if (this.combinators[i + 1] == "obs") {
                    // Keep track of observable index
                    this.combinatorObsIndexMap[i.toString()] = Object.keys(this.combinatorObsIndexMap).length;

                    subCombinatorRes = this._processCombinators(i + 3);
                } else {
                    subCombinatorRes = this._processCombinators(i + 2);
                }

                return subCombinatorRes;

            case "and":
            case "or":
                var subHorizonRes0 = this._processCombinators(i + 1);
                var subHorizonRes1 = this._processCombinators(subHorizonRes0.tailIndex);
                var finalHorizon = this._getMaxHorizon(subHorizonRes0.horizon, subHorizonRes1.horizon);

                // Merge time slices
                var timeSlices = subHorizonRes0.timeSlices;
                timeSlices.merge(subHorizonRes1.timeSlices);

                var anytimeTimeSlices = subHorizonRes0.anytimeTimeSlices.concat(subHorizonRes1.anytimeTimeSlices);

                this.combinatorHorizonMap.add(i.toString(), finalHorizon);

                return new ProcessResult(finalHorizon, subHorizonRes1.tailIndex, timeSlices, anytimeTimeSlices);

            case "then":
                var subHorizonRes0 = this._processCombinators(i + 1);
                var subHorizonRes1 = this._processCombinators(subHorizonRes0.tailIndex);
                var finalHorizon = this._getMaxHorizon(subHorizonRes0.horizon, subHorizonRes1.horizon);

                // Merge time slices, only adding sub-combinator 2's slices after sub-combinator 1's horizon
                var timeSlices = subHorizonRes0.timeSlices;
                timeSlices.mergeAfter(subHorizonRes1.timeSlices);

                var anytimeTimeSlices = subHorizonRes0.anytimeTimeSlices.concat(subHorizonRes1.anytimeTimeSlices);

                this.combinatorHorizonMap.add(i.toString(), finalHorizon);

                return new ProcessResult(finalHorizon, subHorizonRes1.tailIndex, timeSlices, anytimeTimeSlices);
        }
    }

    /**
     * Evaluate the contract using the step-through evaluation options.
     * @param i The index of the combinator to evaluate.
     * @param time The time of acquisition of the sub-contract being evaluated.
     * @param options The list of options for any input-reliant combinators.
     */
    _stepThroughEvaluate(i, time, options) {
        // If this contract has expired, return 0.
        if (this.combinatorHorizonMap.getNextValue(i.toString) < time) {
            return new StepThroughEvaluationResult(0);
        }

        switch (this.combinators[i]) {
            case "zero":
                return new StepThroughEvaluationResult(0);

            case "one":
                return new StepThroughEvaluationResult(1);

            case "give":
                var subRes = this._stepThroughEvaluate(i + 1, time, options);

                // Invert value
                subRes.multiplyByScalar(-1);

                return subRes;

            case "truncate":
                // Already checked for horizon, so can just return sub-result
                return this._stepThroughEvaluate(i + 2, time, options);

            case "scale":
                var subRes;

                if (this.combinatorObsIndexMap[i.toString()]) {
                    // Combinator has observale value
                    subRes = this._stepThroughEvaluate(i + 3, time, options);
                    subRes.addObservableIndex(this.combinatorObsIndexMap[i.toString()]);
                } else {
                    subRes = this._stepThroughEvaluate(i + 2, time, options);

                    var scalar = parseInt(this.combinators[i + 1]);
                    subRes.multiplyByScalar(scalar);
                }

                return subRes;

            case "get":
                // Return the value of the sub-contract at this contract's horizon
                var horizon = this.combinatorHorizonMap[i.toString()];

                return this._stepThroughEvaluate(i + 1, horizon, options);

            case "anytime":
                // Get the anytime acquisition time
                var acquisitionTime = options.shift();

                if (acquisitionTime.type != StepThroughOptions.TYPE_ACQUISITION_TIME) {
                    throw "Expected acquisition time, found or-choice during evaluation.";
                }

                return this._stepThroughEvaluate(i + 1, acquisitionTime, options);

            case "and":
                var subRes0 = this._stepThroughEvaluate(i + 1, time, options);
                var contractEnd = this._getEndOfContract(i + 1);

                var subRes1 = this._stepThroughEvaluate(contractEnd + 1, time, options);
            case "or":
            case "then":
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

    /**
     * Gets the end of the contract starting at i in the array of combinators.
     */
    _getEndOfContract(i) {
        while (i < this.combinators.length) {
            switch (this.combinators[i]) {
                case "and":
                case "or":
                case "then":
                    i = this._getEndOfContract(i + 1);
                    return this._getEndOfContract(i + 1);

                case "zero":
                case "one":
                    return i;

                default:
                    i += 1;
            }
        }

        throw "Invalid contract, does not terminate.";
    }

    /**
     * Resets the Evaluator's state.
     */
    _resetState() {
        this.stepThroughIndex = 0;
        this.stepThroughCombinatorIndex = 0;
        this.stepThroughAcquisitionTimeIndex = 0;
        this.stepThroughOptions = [];
        this.hasNextStep = true;
        this.stepThroughValue = undefined;
        this.combinatorHorizonMap = new NextMap();
        this.combinatorTailIndexMap = new NextMap();
        this.combinatorObsIndexMap = {};
        this.horizon = undefined;
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
 * Class representing an input-reliant combinator's value, which will contain either an acquisition time or an or-choice.
 */
class StepThroughValue {
    /**
     * The acquisition time type value.
     */
    static TYPE_ACQUISITION_TIME = "acquisition-time";

    /**
     * The or-choice type value.
     */
    static TYPE_OR_CHOICE = "or-choice";

    /**
     * The type of the step-through value.
     */
    type;

    /**
     * The value of the step-through value.
     */
    value;

    /**
     * The combinator index of the step-through value.
     */
    combinatorIndex;

    /**
     * Initialises a new instance of this class.
     * @param type The type of the step-through value.
     * @param value The value of the step-through value.
     * @param combinatorIndex The combinator index of the step-through value.
     */
    constructor(type, value, combinatorIndex) {
        this.type = type;
        this.value = value;
        this.combinatorIndex = combinatorIndex;
    }
}

/**
 * Class representing the intermediate result of a step-through evaluation.
 */
class StepThroughEvaluationResult {
    /**
     * The list of observable indexes for observables which are factors in this result.
     */
    obsIndexes = [];

    /**
     * The concrete value of this result, before multiplication by observable values.
     */
    concreteValue;

    /**
     * Initialises a new instance of this class.
     * @param concreteValue The concrete value of evaluation before multiplication by observable values.
     */
    constructor(concreteValue) {
        this.concreteValue = concreteValue;
    }

    /**
     * Multiply the concrete value by a scalar.
     * @param scalar The scalar to multiply the concrete value by.
     */
    multiplyByScalar(scalar) {
        this.concreteValue = this.concreteValue * scalar;
    }

    /**
     * Add an observable index to the list of observables which are factors in the final result.
     * @param obsIndex The observable index.
     */
    addObservableIndex(obsIndex) {
        this.obsIndexes.push(obsIndex);
    }

    /**
     * Adds another step-through evaluation result to this one.
     */
    add(other) {
        this.concreteValue += other.concreteValue;
        this.obsIndexes = this.obsIndexes.concat(other.obsIndexes);
    }
}

/**
 * Maps keys to values, where each key maps to the equal or next-highest key in the keyset (if one exists).
 */
class NextMap {

    /**
     * The mapping of keys to values.
     */
    keyValueMap = {};

    /**
     * Add a value with the given key to the map.
     * @param key The key to store the key under.
     * @param value The value to add.
     */
    add(key, value) {
        keyValueMap[key.toString()] = value;
    }

    /**
     * Get the value stored at the next-highest (or equal) key.
     * @param key The key.
     */
    getNextValue(key) {
        var keys = Object.keys(keyValueMap).sort();
        if (keys.length == 0) {
            return undefined;
        }

        var index = keys.findIndex(elem => elem <= key);
        if (index == -1) {
            return undefined;
        }

        return keyValueMap[keys[index].toString()];
    }
}