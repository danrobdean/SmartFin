import { dateToUnixTimestamp } from "./contract-utils.mjs";
import NextMap from "./next-map.mjs";
import StepThroughOptions from "./step-through-options.mjs";
import TimeSlices from "./time-slices.mjs";

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
     * The current time for step-through execution.
     */
    stepThroughTime;

    /**
     * The stack of AND combinators we've reached while stepping-through the contract which need revisiting.
     */
    stepThroughRevisitAndStack;

    /**
     * The current set of options from stepping through the combinator contract.
     */
    stepThroughValues;

    /**
     * Whether or not the step-through contract evaluation can continue.
     */
    hasMoreSteps;

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
        this._resetStepThroughState();

        // Set initial acquisition time to now if it makes no difference
        var timeSlices = this.timeSlices.getSlices();
        if (timeSlices.length == 0) {
            this.setStepThroughOption(dateToUnixTimestamp(new Date()));
        } else if (timeSlices.length == 1) {
            this.setStepThroughOption(timeSlices[0]);
        }
    }

    /**
     * If the step-through of the contract is concluded, evaluate the value based on the options provided.
     */
    getStepThroughEvaluationValue() {
        if (this.stepThroughValue !== undefined) {
            return this.stepThroughValue;
        }

        var options = this.stepThroughValues.slice();
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
        return this.hasMoreSteps;
    }

    /**
     * Sets the current step-through evaluation option.
     * @param option The input option for the current input-reliant combinator.
     */
    setStepThroughOption(option) {
        // Set the step through option and increment the combinator-index to the start of the next contract
        if (this.stepThroughIndex == 0) {
            // Set the top-level acquisition time
            this.stepThroughValues.push(new StepThroughValue(StepThroughValue.TYPE_ACQUISITION_TIME, option, this.stepThroughCombinatorIndex));

            this.stepThroughCombinatorIndex += 1;
            this.stepThroughAcquisitionTimeIndex += 1;
            this.stepThroughTime = option;
        } else {
            switch (this.combinators[this.stepThroughCombinatorIndex]) {
                case "or":
                    this.stepThroughValues.push(new StepThroughValue(StepThroughValue.TYPE_OR_CHOICE, option, this.stepThroughCombinatorIndex));

                    this.stepThroughCombinatorIndex += 1;

                    // Skip first sub-combinator if option is false, i.e. second child is chosen
                    if (!option) {
                        this.stepThroughCombinatorIndex = this.combinatorTailIndexMap.getNextValue(this.stepThroughCombinatorIndex);
                    }
                    break;

                case "anytime":
                    this.stepThroughValues.push(new StepThroughValue(StepThroughValue.TYPE_ACQUISITION_TIME, option, this.stepThroughCombinatorIndex));

                    // Move on to sub-combinator.
                    this.stepThroughCombinatorIndex += 1;
                    this.stepThroughAcquisitionTimeIndex += 1;
                    this.stepThroughTime = option;
                    break;

                default:
                    throw "Tried to set a non-existent step-through option."
            }
        }

        // Progress to the next input-reliant combinator
        this._goToNextStep();

        return this.hasMoreSteps;
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
        this.stepThroughCombinatorIndex = this.stepThroughValues[index].combinatorIndex;
        this.stepThroughValues = this.stepThroughValues.slice(0, index);

        this.stepThroughAcquisitionTimeIndex = this.stepThroughValues.reduce((prev, cur) => {
            return (cur.type == StepThroughValue.TYPE_ACQUISITION_TIME) ? prev + 1 : prev;
        }, 0);

        var reverseValues = this.stepThroughValues.slice().reverse();
        var timeIndex = reverseValues.findIndex(elem => elem.type == StepThroughValue.TYPE_ACQUISITION_TIME);
        if (timeIndex != -1) {
            this.stepThroughTime = reverseValues[timeIndex].value;
        }

        var andStack = this.stepThroughRevisitAndStack.slice();
        var andSliceIndex = andStack.findIndex(elem => elem.combinatorIndex > this.stepThroughCombinatorIndex);
        this.stepThroughRevisitAndStack = andStack.slice(0, andSliceIndex);

        this.hasMoreSteps = true;

        this.setStepThroughOption(option);
    }

    /**
     * Gets the options for the next step of evaluating the financial contract.
     */
    getNextStepThroughOptions() {
        if (!this.hasMoreSteps) {
            return undefined;
        }

        if (this.stepThroughIndex == 0) {
            var slices = this.timeSlices.getSlices();

            return new StepThroughOptions(StepThroughOptions.TYPE_ACQUISITION_TIME, slices, this.stepThroughCombinatorIndex);
        }

        switch (this.combinators[this.stepThroughCombinatorIndex]) {
            case "or":
                return new StepThroughOptions(StepThroughOptions.TYPE_OR_CHOICE, [true, false], this.stepThroughCombinatorIndex);

            case "anytime":
                // Cut off the time-slice options by the current sub-contract's acquisition time
                var timeSlices = this.anytimeTimeSlices[this.stepThroughAcquisitionTimeIndex - 1].getSlices();
                var index = timeSlices.findIndex(elem => elem >= this.stepThroughTime);
                if (index == -1) {
                    // No time slices, so either expired (not possible here) or makes no difference
                    timeSlices = [];
                } else {
                    timeSlices = timeSlices.slice(index);
                }

                return new StepThroughOptions(StepThroughOptions.TYPE_ACQUISITION_TIME, timeSlices, this.stepThroughCombinatorIndex);

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
                // Add end-of-contract tail index to map
                this.combinatorTailIndexMap.add(i, i + 1);

                return new ProcessResult(undefined, new TimeSlices(), []);

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

                return new ProcessResult(finalHorizon, timeSlices, subHorizonRes.anytimeTimeSlices);

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
            case "then":
                var subHorizonRes0 = this._processCombinators(i + 1);
                var tailIndex0 = this.combinatorTailIndexMap.getNextValue(i + 1);

                var subHorizonRes1 = this._processCombinators(tailIndex0);
                var tailIndex1 = this.combinatorTailIndexMap.getNextValue(tailIndex0);

                var finalHorizon = this._getMaxHorizon(subHorizonRes0.horizon, subHorizonRes1.horizon);

                // Merge time slices
                var timeSlices = subHorizonRes0.timeSlices;
                if (this.combinators[i] == "then") {
                    // Merge time slices, only adding sub-combinator 2's slices after sub-combinator 1's horizon
                    timeSlices.mergeAfter(subHorizonRes1.timeSlices);
                } else {
                    timeSlices.merge(subHorizonRes1.timeSlices);
                }

                var anytimeTimeSlices = subHorizonRes0.anytimeTimeSlices.concat(subHorizonRes1.anytimeTimeSlices);

                this.combinatorHorizonMap.add(i, finalHorizon);
                this.combinatorTailIndexMap.add(i, tailIndex1);

                return new ProcessResult(finalHorizon, timeSlices, anytimeTimeSlices);
        }
    }

    _goToNextStep() {
        // Increment step-through index to next input-reliant combinator
        this.stepThroughIndex += 1;

        var foundValidInputReliantCombinator = false;

        while (this.hasMoreSteps && !foundValidInputReliantCombinator) {
            switch (this.combinators[this.stepThroughCombinatorIndex]) {
                case "and":
                    this.stepThroughRevisitAndStack.push(new StepThroughRevisitAndEntry(this.stepThroughCombinatorIndex, this.stepThroughTime));
                    this.stepThroughCombinatorIndex += 1;

                    break;

                case "then":
                    if (this._horizonLaterThan(this.stepThroughTime, this.combinatorHorizonMap.tryGetNextValue(this.stepThroughCombinatorIndex + 1))) {
                        this.stepThroughCombinatorIndex = this.combinatorTailIndexMap.getNextValue(this.stepThroughCombinatorIndex + 1);
                    } else {
                        this.stepThroughCombinatorIndex += 1;
                    }

                    break;
                
                case "or":
                case "anytime":
                    if (this._horizonLaterThan(this.stepThroughTime, this.combinatorHorizonMap.tryGetNextValue(this.stepThroughCombinatorIndex))) {
                        // Combinator has expired, don't bother setting it and move on
                        this._tryRevisitOrEndStepThrough();
                    } else {
                        if (this.combinators[this.stepThroughCombinatorIndex] == "or") {
                            // OR combinator, check if either sub-contract has expired, if so then make choice and move on
                            var subHorizon0 = this.combinatorHorizonMap.tryGetNextValue(this.stepThroughCombinatorIndex + 1);
                            var tail0 = this.combinatorTailIndexMap.getNextValue(this.stepThroughCombinatorIndex + 1);

                            var subHorizon1 = this.combinatorHorizonMap.tryGetNextValue(tail0);

                            if (this._horizonLaterThan(this.stepThroughTime, subHorizon0)) {
                                // First sub-contract expired, make choice for second sub-combinator
                                return this.setStepThroughOption(false);
                            } else if (this._horizonLaterThan(this.stepThroughTime, subHorizon1)) {
                                // Second sub-contract expired, make choice for first sub-combinator
                                return this.setStepThroughOption(true);
                            }
                        } else {
                            // ANYTIME combinator, check if only one acquisition time, in which case make choice
                            var timeSlices = this.anytimeTimeSlices[this.stepThroughAcquisitionTimeIndex - 1].getSlices();
                            var index = timeSlices.findIndex(elem => elem >= this.stepThroughTime);

                            if (index == -1) {
                                timeSlices = [];
                            } else {
                                timeSlices = timeSlices.slice(index);
                            }

                            if (timeSlices.length == 0) {
                                // No choices to be made, make choice and move on
                                return this.setStepThroughOption(this.stepThroughTime);
                            } else if (timeSlices.length == 1) {
                                return this.setStepThroughOption(timeSlices[0]);
                            }
                        }
                        foundValidInputReliantCombinator = true;
                    }

                    break;

                case "get":
                    // Update time to horizon of GET if it's greater than the current time, then fall through
                    var horizon = this.combinatorHorizonMap.tryGetNextValue(this.stepThroughCombinatorIndex);
                    if (horizon === undefined) {
                        // Get combinator will never be acquired, so no need to continue setting options
                        this._tryRevisitOrEndStepThrough();
                    } else {
                        this.stepThroughTime = this._getMaxHorizon(this.stepThroughTime, horizon);
                    }

                    break;

                default:
                    // If we've reached a combinator with no children, or the horizon has
                    // passed and the sub-contract is worthless, the sub-contract has ended
                    var subContractEnd =
                        ["zero", "one"].includes(this.combinators[this.stepThroughCombinatorIndex])
                        || this._horizonLaterThan(this.stepThroughTime, this.combinatorHorizonMap.tryGetNextValue(this.stepThroughCombinatorIndex));

                    if (!subContractEnd) {
                        this.stepThroughCombinatorIndex += 1;
                    } else {
                        this._tryRevisitOrEndStepThrough();
                    }

                    break;
            }
        }
    }

    /**
     * If ANDs need to be revisited while stepping through the contract, revisit them, otherwise stop.
     */
    _tryRevisitOrEndStepThrough() {
        if (this.stepThroughRevisitAndStack.length == 0) {
            // End of contract
            this.hasMoreSteps = false;
        } else {
            // Need to visit other and branch, pop details from stack and move to next index at correct time
            var revisitAndEntry = this.stepThroughRevisitAndStack.pop();
            this.stepThroughTime = revisitAndEntry.time;
            this.stepThroughCombinatorIndex = this.combinatorTailIndexMap.getNextValue(revisitAndEntry.combinatorIndex + 1);
        }
    }

    /**
     * Evaluate the contract using the step-through evaluation options.
     * @param i The index of the combinator to evaluate.
     * @param time The time of acquisition of the sub-contract being evaluated.
     * @param values The list of values for any input-reliant combinators.
     */
    _stepThroughEvaluate(i, time, values) {
        // If this contract has expired, return 0.
        if (this._horizonLaterThan(time, this.combinatorHorizonMap.tryGetNextValue(i))) {
            return new StepThroughEvaluationResult(0);
        }

        switch (this.combinators[i]) {
            case "zero":
                return new StepThroughEvaluationResult(0);

            case "one":
                return new StepThroughEvaluationResult(1);

            case "give":
                var subRes = this._stepThroughEvaluate(i + 1, time, values);

                // Invert value
                subRes.multiplyByScalar(-1);

                return subRes;

            case "truncate":
                // Already checked for horizon, so can just return sub-result
                return this._stepThroughEvaluate(i + 2, time, values);

            case "scale":
                var subRes;

                if (this.combinatorObsIndexMap[i.toString()]) {
                    // Combinator has observale value
                    subRes = this._stepThroughEvaluate(i + 3, time, values);
                    subRes.addObservableIndex(this.combinatorObsIndexMap[i.toString()]);
                } else {
                    subRes = this._stepThroughEvaluate(i + 2, time, values);

                    var scalar = parseInt(this.combinators[i + 1]);
                    subRes.multiplyByScalar(scalar);
                }

                return subRes;

            case "get":
                // Return the value of the sub-contract at this contract's horizon
                var horizon = this.combinatorHorizonMap.tryGetNextValue(i);

                return this._stepThroughEvaluate(i + 1, horizon, values);

            case "anytime":
                // Get the anytime acquisition time
                var acquisitionTime = values.shift();

                if (acquisitionTime.type != StepThroughValue.TYPE_ACQUISITION_TIME) {
                    throw "Expected acquisition time, found or-choice during evaluation.";
                }

                return this._stepThroughEvaluate(i + 1, acquisitionTime.value, values);

            case "and":
                var subRes0 = this._stepThroughEvaluate(i + 1, time, values);
                var tail0 = this.combinatorTailIndexMap.getNextValue(i + 1);

                var subRes1 = this._stepThroughEvaluate(tail0, time, values);
                return subRes0.add(subRes1);

            case "or":
                // Get the or-choice
                var orChoice = values.shift();

                if (orChoice.type != StepThroughValue.TYPE_OR_CHOICE) {
                    throw "Expected or-choice, found acquisition time during evaluation.";
                }

                var nextI = i + 1;
                if (!orChoice) {
                    nextI = this.combinatorTailIndexMap.getNextValue(nextI);
                }

                return this._stepThroughEvaluate(nextI, time, values);

            case "then":
                var nextI = i + 1;

                // If first sub-contract expired, check second sub-contract
                if (this._horizonLaterThan(time, this.combinatorHorizonMap.tryGetNextValue(nextI))) {
                    nextI = this.combinatorTailIndexMap.getNextValue(nextI);
                }

                return this._stepThroughEvaluate(nextI, time, values);

            default:
                throw "Expected combinator, found '" + this.combinators[i] + "'.";
        }
    }

    /**
     * Check whether the first given horizon is later than the second given horizon.
     * @param h0 The first horizon.
     * @param h1 The second horizon.
     */
    _horizonLaterThan(h0, h1) {
        if (h1 === undefined) {
            return false;
        } else if (h0 === undefined) {
            return true;
        } else {
            return h0 > h1;
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
     * Resets the Evaluator's state.
     */
    _resetState() {
        this.combinatorHorizonMap = new NextMap();
        this.combinatorTailIndexMap = new NextMap();
        this.combinatorObsIndexMap = {};
        this.horizon = undefined;

        this._resetStepThroughState();
    }

    /**
     * Resets the Evaluator's state for step-through evaluation.
     */
    _resetStepThroughState() {
        this.stepThroughIndex = 0;
        this.stepThroughCombinatorIndex = -1;
        this.stepThroughAcquisitionTimeIndex = 0;
        this.stepThroughTime = undefined;
        this.stepThroughRevisitAndStack = [];
        this.stepThroughValues = [];
        this.hasMoreSteps = true;
        this.stepThroughValue = undefined;
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
     * @param timeSlices The TimeSlices of the contract.
     * @param anytimeTimeSlices The array of TimeSlices of the anytime combinators in the contract.
     */
    constructor(horizon, timeSlices, anytimeTimeSlices) {
        this.horizon = horizon;
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
 * Class representing an entry into the revisit AND stack, where AND combinators
 * must be revisited to set the options of both sub-contracts.
 */
class StepThroughRevisitAndEntry {
    /**
     * The combinator index of the AND combinator to revisit.
     */
    combinatorIndex;

    /**
     * The time to revisit the AND combinator at.
     */
    time;

    /**
     * Initialises a new instance of this class.
     * @param combinatorIndex The combinator index of the AND combinator.
     * @param time The time to revisit the AND combinator at.
     */
    constructor(combinatorIndex, time) {
        this.combinatorIndex = combinatorIndex;
        this.time = time;
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