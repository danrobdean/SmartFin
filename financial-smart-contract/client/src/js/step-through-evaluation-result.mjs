/**
 * Class representing the intermediate result of a step-through evaluation.
 */
export default class StepThroughEvaluationResult {
    /**
     * A stack of intermediate evaluation results.
     */
    intermediateResultStack = [];

    /**
     * Initiailises a new instance of this class, with the given payment value.
     * @param value The payment value.
     * @param acquisitionTimeSlice The time slice that this result is acquired in.
     */
    constructor(value, acquisitionTimeSlice) {
        this.intermediateResultStack.push(new IntermediateResult(IntermediateResult.TYPE_PAYMENT, value, acquisitionTimeSlice));
    }

    /**
     * Gets a string representation of the value of this result.
     * @param showTimes Whether or not to include acquisition times in the value string.
     */
    getValue(showTimes) {
        var res = this._formatResult(showTimes, 1, this.intermediateResultStack.slice());

        if (res === undefined) {
            return "0 Wei";
        } else {
            return res;
        }
    }

    /**
     * Multiply the concrete value by a scalar.
     * @param scalar The scalar to multiply the concrete value by.
     */
    multiplyByScalar(scalar) {
        this.intermediateResultStack.push(new IntermediateResult(IntermediateResult.TYPE_SCALE, scalar));
    }

    /**
     * Add an observable index to the list of observables which are factors in the final result.
     * @param obsName The observable name.
     * @param acquisitionTimeSlice The time slice that the observable will be queried in.
     */
    addObservable(obsName, acquisitionTimeSlice) {
        this.intermediateResultStack.push(new IntermediateResult(IntermediateResult.TYPE_OBSERVABLE, obsName, acquisitionTimeSlice));
    }

    /**
     * Adds another step-through evaluation result to this one.
     */
    add(other) {
        this.intermediateResultStack = other.intermediateResultStack.concat(this.intermediateResultStack);

        this.intermediateResultStack.push(new IntermediateResult(IntermediateResult.TYPE_ADD));
    }

    /**
     * Consumes the intermediate result stack until a payment is reached, and returns a string representing the overall payment value.
     * @param showTimes Whether or not to show acquisition times in the result string.
     * @param scalar The scalar to multiply the payment value by.
     * @param intermediateResultStack The intermediate result stack to process.
     */
    _formatResult(showTimes, scalar, intermediateResultStack) {
        var intermediate = intermediateResultStack.pop();

        switch (intermediate.type) {
            case IntermediateResult.TYPE_PAYMENT:
                var value = intermediate.value;
                value *= scalar;

                if (value == 0) {
                    return undefined;
                }

                value = value.toString() + " Wei";
                if (showTimes) {
                    value += " <" + intermediate.acquisitionTimeSlice.toDateRangeString() + ">";
                }
                return value;

            case IntermediateResult.TYPE_SCALE:
                return this._formatResult(showTimes, scalar * intermediate.value, intermediateResultStack);

            case IntermediateResult.TYPE_OBSERVABLE:
                var subRes = this._formatResult(showTimes, scalar, intermediateResultStack);
                if (subRes === undefined) {
                    return subRes;
                }
                
                var value =  intermediate.value;
                if (showTimes) {
                    value += " <" + intermediate.acquisitionTimeSlice.toDateRangeString() + ">";
                }
                value += " * " + subRes;
                return value;

            case IntermediateResult.TYPE_ADD:
                var subRes0 = this._formatResult(showTimes, scalar, intermediateResultStack);
                var subRes1 = this._formatResult(showTimes, scalar, intermediateResultStack);
                if (subRes0 === undefined) {
                    return subRes1;
                } else if (subRes1 === undefined) {
                    return subRes0;
                }

                return "(" + subRes0 + " + " + subRes1 + ")";

            default:
                throw "Unrecognised intermediate step-through evaluation result found: " + intermediate.type;
        }
    }
}

/**
 * An intermediate step-through evaluation result.
 */
class IntermediateResult {
    /**
     * The string representing the payment type.
     */
    static TYPE_PAYMENT = "TYPE_PAYMENT";

    /**
     * The string representing the scale type.
     */
    static TYPE_SCALE = "TYPE_SCALE";

    /**
     * The string representing the observable type.
     */
    static TYPE_OBSERVABLE = "TYPE_OBSERVABLE";

    /**
     * The string representing the add marker type.
     */
    static TYPE_ADD = "TYPE_ADD";

    /**
     * The type of this intermediate result.
     */
    type;

    /**
     * The value of this intermediate result (scalar value, payment value, or observable name)
     */
    value;

    /**
     * The time slice in which this intermediate result's corresponding combinator was acquired in.
     */
    acquisitionTimeSlice;

    /**
     * Initialises a new instance of this class.
     * @param type The type of this intermediate result.
     * @param value The value of this intermediate result (scalar value, payment value, or observable name).
     * @param acquisitionTimeSlice The time slice in which this intermediate result's corresponding combinator was acquired in.
     */
    constructor(type, value, acquisitionTimeSlice) {
        this.type = type;
        this.value = value;
        this.acquisitionTimeSlice = acquisitionTimeSlice;
    }
}