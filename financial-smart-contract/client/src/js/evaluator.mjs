import moment from "moment";

import { UNIX_FORMAT, DATE_STRING_FORMAT, DATE_STRING_NO_ZONE_FORMAT, compareTime, splitContract } from "./contract-utils.mjs";
import NextMap from "./next-map.mjs";
import StepThroughEvaluationResult from "./step-through-evaluation-result.mjs";
import StepThroughOptions from "./step-through-options.mjs";
import StepThroughValue from "./step-through-value.mjs";
import TimeSlice from "./time-slice.mjs";
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
     * The current or index from stepping through the combinators in the contract.
     */
    stepThroughOrIndex;

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
        this._resetState();
        this._resetStepThroughState();
        this.contract = contract;

        // Set the array of combinator atoms.
        // Split by any character of '(', ')', ',', or ' '
        this.combinators = splitContract(this.contract);

        var result = this._processCombinators(0);
        this.horizon = result.horizon;
        this.timeSlices = result.timeSlices;
        this.anytimeTimeSlices = result.anytimeTimeSlices;
    }

    /**
     * If the step-through of the contract is concluded, evaluate the value based on the options provided.
     * @param showTimes Whether or not to show acquisition times in the result string.
     */
    evaluate(showTimes) {
        if (!this.contract) {
            throw "Can't evaluate with no defined contract.";
        }

        if (this.stepThroughValue !== undefined) {
            return this.stepThroughValue;
        }

        if (this.hasNextStep()) {
            // Step-through not done
            return undefined;
        }

        var options = this.stepThroughValues.slice();
        var time = options.shift().value;
        var res = this._stepThroughEvaluate(0, time, options);

        return res.getValue(showTimes);
    }

    /**
     * Returns true if more step-through options can be set, false if otherwise.
     */
    hasNextStep() {
        return this.hasMoreSteps;
    }

    /**
     * Returns the set of step-through values which have already been set.
     */
    getPrevValues() {
        if (this.stepThroughValues) {
            return this.stepThroughValues;
        }
    }

    /**
     * Sets the current step-through evaluation option.
     * @param option The input option for the current input-reliant combinator.
     */
    setStepThroughOption(option) {
        if (!this.contract) {
            throw "Can't evaluate with no defined contract.";
        }

        // Set the step through option and increment the combinator-index to the start of the next contract
        if (this.stepThroughIndex == 0) {
            // Set the top-level acquisition time
            this.stepThroughValues.push(new StepThroughValue(StepThroughValue.TYPE_ACQUISITION_TIME, option, this.stepThroughCombinatorIndex, -1));

            this.stepThroughCombinatorIndex += 1;
            this.stepThroughAcquisitionTimeIndex += 1;
            this.stepThroughTime = option;
        } else {
            switch (this.combinators[this.stepThroughCombinatorIndex].toLowerCase()) {
                case "or":
                    this.stepThroughValues.push(new StepThroughValue(StepThroughValue.TYPE_OR_CHOICE, option, this.stepThroughCombinatorIndex, this.stepThroughOrIndex));

                    this.stepThroughCombinatorIndex += 1;

                    // Skip first sub-combinator if option is false, i.e. second child is chosen
                    if (!option) {
                        this.stepThroughCombinatorIndex = this.combinatorTailIndexMap.getNextValue(this.stepThroughCombinatorIndex);
                    }

                    this.stepThroughOrIndex += 1;
                    break;

                case "anytime":
                    this.stepThroughValues.push(new StepThroughValue(StepThroughValue.TYPE_ANYTIME_ACQUISITION_TIME, option, this.stepThroughCombinatorIndex, this.stepThroughAcquisitionTimeIndex - 1));

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
     * @param combinatorIndex The combinator index of the step-through option to reset.
     */
    deleteStepThroughOption(combinatorIndex) {
        if (!this.contract) {
            throw "Can't evaluate with no defined contract.";
        }

        if (this.stepThroughCombinatorIndex <= combinatorIndex) {
            throw "Tried to reset a step-through option which has not yet been set.";
        }

        var stepThroughIndex = this.stepThroughValues.findIndex(elem => elem.combinatorIndex == combinatorIndex);
        if (stepThroughIndex == -1) {
            throw "Tried to reset a step-through option which does not exist.";
        }

        this.stepThroughIndex = stepThroughIndex;
        this.stepThroughCombinatorIndex = combinatorIndex;
        this.stepThroughValues = this.stepThroughValues.slice(0, this.stepThroughIndex);

        this.stepThroughAcquisitionTimeIndex = this.stepThroughValues.reduce((prev, cur) => {
            return ([StepThroughValue.TYPE_ACQUISITION_TIME, StepThroughValue.TYPE_ANYTIME_ACQUISITION_TIME].includes(cur.type)) ? prev + 1 : prev;
        }, 0);

        var reverseValues = this.stepThroughValues.slice().reverse();
        var timeIndex = reverseValues.findIndex(elem => [StepThroughValue.TYPE_ACQUISITION_TIME, StepThroughValue.TYPE_ANYTIME_ACQUISITION_TIME, StepThroughValue.TYPE_GET_ACQUISITION_TIME].includes(elem.type));
        if (timeIndex != -1) {
            this.stepThroughTime = reverseValues[timeIndex].value;
        }

        var andStack = this.stepThroughRevisitAndStack.slice();
        var andSliceIndex = andStack.findIndex(elem => elem.combinatorIndex > this.stepThroughCombinatorIndex);
        this.stepThroughRevisitAndStack = andStack.slice(0, andSliceIndex);

        this.hasMoreSteps = true;
    }

    /**
     * Gets the options for the next step of evaluating the financial contract.
     * @param includePast Whether or not to return acquisition time options in the past.
     */
    getNextStepThroughOptions(includePast) {
        if (!this.contract) {
            throw "Can't evaluate with no defined contract.";
        }

        if (!this.hasMoreSteps) {
            return undefined;
        }

        if (this.stepThroughIndex == 0) {
            var unixNow = moment().unix();
            var slices = (includePast) ? this.timeSlices.getSlices() : this.timeSlices.getValidSlices(unixNow);
            var horizon = this.timeSlices.getEndTime();

            // Add option at current time/after horizon (if one exists)
            if (!includePast && this._horizonLaterThan(unixNow, horizon)) {
                slices.push(new TimeSlice(unixNow, undefined));
            } else if (horizon !== undefined) {
                slices.push(new TimeSlice(horizon + 1, undefined));
            }

            return new StepThroughOptions(StepThroughOptions.TYPE_ACQUISITION_TIME, slices, this.stepThroughCombinatorIndex, -1);
        }

        switch (this.combinators[this.stepThroughCombinatorIndex].toLowerCase()) {
            case "or":
                return new StepThroughOptions(StepThroughOptions.TYPE_OR_CHOICE, [true, false], this.stepThroughCombinatorIndex, this.stepThroughOrIndex);

            case "anytime":
                // Cut off the time-slice options by the current sub-contract's acquisition time
                var timeSlices = this.anytimeTimeSlices[this.stepThroughAcquisitionTimeIndex - 1].getValidSlices(this.stepThroughTime.getStart());

                return new StepThroughOptions(StepThroughOptions.TYPE_ANYTIME_ACQUISITION_TIME, timeSlices, this.stepThroughCombinatorIndex, this.stepThroughAcquisitionTimeIndex - 1);

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
        if (!this.contract) {
            throw "Can't evaluate with no defined contract.";
        }

        return this.horizon;
    }

    /**
     * Gets the set of relevant time slices for the contract.
     */
    getTimeSlices() {
        if (!this.contract) {
            throw "Can't evaluate with no defined contract.";
        }

        return this.timeSlices.clone();
    }

    /**
     * Gets the array of anytime time slice sets for the contract, in order of occurrence.
     */
    getAnytimeTimeSlices() {
        if (!this.contract) {
            throw "Can't evaluate with no defined contract.";
        }

        return this.anytimeTimeSlices;
    }

    /**
     * Gets the horizon of the given financial contract, or undefined if none exists.
     * @param i The index to start processing the associated combinator contract at.
     */
    _processCombinators(i) {
        switch (this.combinators[i].toLowerCase()) {
            case "zero":
            case "one":
                // Add horizon to map
                this.combinatorHorizonMap.add(i, undefined);

                // Add end-of-contract tail index to map
                this.combinatorTailIndexMap.add(i, i + 1);

                return new ProcessResult(undefined, new TimeSlices(), []);

            case "truncate":
                var horizon;
                var subHorizonRes;

                if (this.combinators[i + 1].indexOf("<") != -1) {
                    // Time is pretty date string, find closing bracket
                    var closeIndex = this.combinators.slice(i + 1).findIndex(elem => elem.indexOf(">") != -1) + i + 1;

                    horizon = this._dateToUnix(this.combinators.slice(i + 1, closeIndex + 1).join(" "));
                    subHorizonRes = this._processCombinators(closeIndex + 1);
                } else {
                    // Time is unix
                    horizon = moment.utc(this.combinators[i + 1], UNIX_FORMAT, true).unix();
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
                subHorizonRes.timeSlices.concatHead(subHorizonRes.horizon);

                return subHorizonRes;

            case "anytime":
                var subHorizonRes = this._processCombinators(i + 1);

                // Add current contract's time slices to front of anytime time slices array.
                subHorizonRes.anytimeTimeSlices.unshift(subHorizonRes.timeSlices.clone());

                return subHorizonRes;

            case "scale":
                var subCombinatorRes;

                if (isNaN(this.combinators[i + 1])) {
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
                if (this.combinators[i].toLowerCase() == "then") {
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
            switch (this.combinators[this.stepThroughCombinatorIndex].toLowerCase()) {
                case "and":
                    this.stepThroughRevisitAndStack.push(new StepThroughRevisitAndEntry(this.stepThroughCombinatorIndex, this.stepThroughTime));
                    this.stepThroughCombinatorIndex += 1;

                    break;

                case "then":
                    if (this._horizonLaterThan(this.stepThroughTime.getStart(), this.combinatorHorizonMap.tryGetNextValue(this.stepThroughCombinatorIndex + 1))) {
                        this.stepThroughCombinatorIndex = this.combinatorTailIndexMap.getNextValue(this.stepThroughCombinatorIndex + 1);
                    } else {
                        this.stepThroughCombinatorIndex += 1;
                    }

                    break;
                
                case "or":
                case "anytime":
                    if (this._horizonLaterThan(this.stepThroughTime.getStart(), this.combinatorHorizonMap.tryGetNextValue(this.stepThroughCombinatorIndex))) {
                        // Combinator has expired, don't bother setting it and move on
                        this._tryRevisitOrEndStepThrough();
                    } else {
                        if (this.combinators[this.stepThroughCombinatorIndex].toLowerCase() == "or") {
                            // OR combinator, check if either sub-contract has expired, if so then make choice and move on
                            var subHorizon0 = this.combinatorHorizonMap.tryGetNextValue(this.stepThroughCombinatorIndex + 1);
                            var tail0 = this.combinatorTailIndexMap.getNextValue(this.stepThroughCombinatorIndex + 1);

                            var subHorizon1 = this.combinatorHorizonMap.tryGetNextValue(tail0);

                            if (this._horizonLaterThan(this.stepThroughTime.getStart(), subHorizon0)) {
                                // First sub-contract expired, make choice for second sub-combinator
                                return this.setStepThroughOption(false);
                            } else if (this._horizonLaterThan(this.stepThroughTime.getStart(), subHorizon1)) {
                                // Second sub-contract expired, make choice for first sub-combinator
                                return this.setStepThroughOption(true);
                            }
                        } else {
                            // ANYTIME combinator, check if only one acquisition time, in which case make choice
                            var timeSlices = this.anytimeTimeSlices[this.stepThroughAcquisitionTimeIndex - 1].getValidSlices(this.stepThroughTime.getStart());
                            var index = timeSlices.findIndex(elem => this._horizonLaterThan(elem.getEnd(), this.stepThroughTime.getStart()));

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
                    // Update time to horizon of GET if it's greater than the current time
                    var horizon = this.combinatorHorizonMap.tryGetNextValue(this.stepThroughCombinatorIndex);
                    if (horizon === undefined || this._horizonLaterThan(this.stepThroughTime.getStart(), horizon)) {
                        // Get combinator will never be acquired, so no need to continue setting options
                        this._tryRevisitOrEndStepThrough();
                    } else {
                        // Set current time slice to instant at horizon
                        this.stepThroughTime = new TimeSlice(horizon, horizon);

                        // Store step through time in step through values
                        this.stepThroughValues.push(new StepThroughValue(StepThroughValue.TYPE_GET_ACQUISITION_TIME, this.stepThroughTime, this.stepThroughCombinatorIndex));

                        this.stepThroughCombinatorIndex += 1;
                    }

                    break;

                case "scale":
                    // Skip over any named observables, in case the name is also a combinator name
                    if (isNaN(this.combinators[this.stepThroughCombinatorIndex + 1])) {
                        this.stepThroughCombinatorIndex += 3;
                        break;
                    }

                default:
                    // If we've reached a combinator with no children, or the horizon has
                    // passed and the sub-contract is worthless, the sub-contract has ended
                    var subContractEnd =
                        ["zero", "one"].includes(this.combinators[this.stepThroughCombinatorIndex].toLowerCase())
                        || this._horizonLaterThan(this.stepThroughTime.getStart(), this.combinatorHorizonMap.tryGetNextValue(this.stepThroughCombinatorIndex));

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
        if (this._horizonLaterThan(time.getStart(), this.combinatorHorizonMap.tryGetNextValue(i))) {
            return new StepThroughEvaluationResult(0, time);
        }

        switch (this.combinators[i].toLowerCase()) {
            case "zero":
                return new StepThroughEvaluationResult(0, time);

            case "one":
                return new StepThroughEvaluationResult(1, time);

            case "give":
                var subRes = this._stepThroughEvaluate(i + 1, time, values);

                // Invert value
                subRes.multiplyByScalar(-1);

                return subRes;

            case "truncate":
                // Already checked for horizon, so can just return sub-result
                if (this.combinators[i + 1].indexOf("<") != -1) {
                    var closeIndex = this.combinators.slice(i + 1).findIndex(elem => elem.indexOf(">") != -1) + i + 1;
                    
                    return this._stepThroughEvaluate(closeIndex + 1, time, values);
                } else {
                    return this._stepThroughEvaluate(i + 2, time, values);
                }

            case "scale":
                var subRes;

                if (isNaN(this.combinators[i + 1])) {
                    // Combinator has observable value
                    subRes = this._stepThroughEvaluate(i + 3, time, values);
                    subRes.addObservable(this.combinators[i + 1], time);
                } else {
                    subRes = this._stepThroughEvaluate(i + 2, time, values);

                    var scalar = parseInt(this.combinators[i + 1]);
                    subRes.multiplyByScalar(scalar);
                }

                return subRes;

            case "get":
                // Return the value of the sub-contract at this contract's horizon
                var horizon = this.combinatorHorizonMap.tryGetNextValue(i);
                if (horizon === undefined) {
                    return new StepThroughEvaluationResult(0);
                } else {
                    return this._stepThroughEvaluate(i + 1, values.shift().value, values);
                }

            case "anytime":
                // Get the anytime acquisition time
                var acquisitionTime = values.shift();

                if (acquisitionTime.type != StepThroughValue.TYPE_ANYTIME_ACQUISITION_TIME) {
                    throw "Expected acquisition time, found or-choice during evaluation.";
                }

                return this._stepThroughEvaluate(i + 1, acquisitionTime.value, values);

            case "and":
                var subRes0 = this._stepThroughEvaluate(i + 1, time, values);
                var tail0 = this.combinatorTailIndexMap.getNextValue(i + 1);

                var subRes1 = this._stepThroughEvaluate(tail0, time, values);

                subRes0.add(subRes1);

                return subRes0;

            case "or":
                // Get the or-choice
                var orChoice = values.shift();

                if (orChoice.type != StepThroughValue.TYPE_OR_CHOICE) {
                    throw "Expected or-choice, found acquisition time during evaluation.";
                }

                var nextI = i + 1;
                if (!orChoice.value) {
                    nextI = this.combinatorTailIndexMap.getNextValue(nextI);
                }

                return this._stepThroughEvaluate(nextI, time, values);

            case "then":
                var nextI = i + 1;

                // If first sub-contract expired, check second sub-contract
                if (this._horizonLaterThan(time.getStart(), this.combinatorHorizonMap.tryGetNextValue(nextI))) {
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
        return compareTime(h0, h1) > 0;
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
    _dateToUnix(date) {
        // If date is angle-bracketed, remove them
        if (date.indexOf("<") != -1) {
            date = date.slice(1, -1);
        }

        // Validate and format to unix time
        date = moment.utc(date, [DATE_STRING_FORMAT, DATE_STRING_NO_ZONE_FORMAT], true);

        // Return date as int, not string
        return date.unix();
    }

    /**
     * Resets the Evaluator's state.
     */
    _resetState() {
        this.combinatorHorizonMap = new NextMap();
        this.combinatorTailIndexMap = new NextMap();
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
        this.stepThroughOrIndex = 0;
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