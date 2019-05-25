/**
 * Class representing a set of options for an input-reliant combinator.
 */
export default class StepThroughOptions {
    /**
     * The acquisition time type value.
     */
    static TYPE_ACQUISITION_TIME = "acquisition-time";

    /**
     * The anytime acquisition time type value.
     */
    static TYPE_ANYTIME_ACQUISITION_TIME = "anytime-acquisition-time";

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
     * The combinator's index in the contract definition.
     */
    combinatorIndex;

    /**
     * The index of that combinator's type.
     */
    index;

    /**
     * Initialises a new instance of this class.
     * @param type The type of the step-through options.
     * @param options The options for the value of the step-through option.
     * @param combinatorIndex The combinator index of the step-through option.
     * @param index The index of that combinator's type.
     */
    constructor(type, options, combinatorIndex, index) {
        this.type = type;
        this.options = options;
        this.combinatorIndex = combinatorIndex;
        this.index = index;
    }
}