/**
 * Class representing a set of options for an input-reliant combinator.
 */
export default class StepThroughOptions {
    /**
     * The acquisition time type value.
     */
    static TYPE_ACQUISITION_TIME = "acquisition-time";

    /**
     * The or-choice type value.
     */
    static TYPE_OR_CHOICE = "or-choice";

    /**
     * The type of the step-through option.
     */
    type;

    /**
     * The options of the step-through option.
     */
    options;

    /**
     * Initialises a new instance of this class.
     * @param type The type of the step-through options.
     * @param options The options for the value of the step-through option.
     * @param combinatorIndex The combinator index of the step-through option.
     */
    constructor(type, options, combinatorIndex) {
        this.type = type;
        this.options = options;
        this.combinatorIndex = combinatorIndex;
    }

    getType() {
        return this.type;
    }

    getOptions() {
        return this.options;
    }
}