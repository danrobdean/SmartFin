/**
 * Class representing an input-reliant combinator's value, which will contain either an acquisition time or an or-choice.
 */
export default class StepThroughValue {
    /**
     * The acquisition time type value.
     */
    static TYPE_ACQUISITION_TIME = "acquisition-time";

    /**
     * The acquisition time type value.
     */
    static TYPE_ANYTIME_ACQUISITION_TIME = "anytime-acquisition-time";

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
     * The index of that combinator's type.
     */
    index;

    /**
     * Whether or not this value was set automatically.
     */
    setAutomatically;

    /**
     * Initialises a new instance of this class.
     * @param type The type of the step-through value.
     * @param value The value of the step-through value.
     * @param combinatorIndex The combinator index of the step-through value.
     * @param index The index of that combinator's type.
     * @param setAutomatically Whether or not this value was set automatically.
     */
    constructor(type, value, combinatorIndex, index, setAutomatically) {
        this.type = type;
        this.value = (type === StepThroughValue.TYPE_ACQUISITION_TIME) ? parseInt(value) : value;
        this.combinatorIndex = combinatorIndex;
        this.index = index;
        this.setAutomatically = setAutomatically;
    }
}