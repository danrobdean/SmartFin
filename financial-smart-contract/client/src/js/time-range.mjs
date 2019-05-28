import { compareTime } from "./contract-utils.mjs";

/**
 * Represents a single discrete range between two times (inclusive).
 */
export default class TimeRange {
    /**
     * The start of the time range.
     */
    start;

    /**
     * The end of the time range.
     */
    end;

    /**
     * Initialises a new instance of this class.
     * @param start The start of the time range.
     * @param end The end of the time range.
     */
    constructor(start, end) {
        if (compareTime(start, end) > 0) {
            throw "Attempted to create a time range with a start after its end.";
        }

        this.start = start;
        this.end = end;
    }

    /**
     * Compares the ends of two ranges.
     */
    static compareRangeEnds(a, b) {
        return compareTime(a.getEnd(), b.getEnd());
    }

    /**
     * Gets the time-range end time.
     */
    getEnd() {
        return this.end;
    }

    /**
     * Gets the time-range start time.
     */
    getStart() {
        return this.start;
    }

    /**
     * Returns true if the given time is within this time range, false otherwise.
     */
    includes(time) {
        return compareTime(this.start, time) <= 0 && compareTime(this.end, time) >= 0;
    }
}