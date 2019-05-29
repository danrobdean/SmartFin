import moment from "moment";

import { compareTime, UNIX_FORMAT, DATE_STRING_FORMAT } from "./contract-utils.mjs";

/**
 * Represents a single discrete range between two times (inclusive).
 */
export default class TimeSlice {
    /**
     * The start of the time slice.
     */
    start;

    /**
     * The end of the time slice.
     */
    end;

    /**
     * Initialises a new instance of this class.
     * @param start The start of the time slice.
     * @param end The end of the time slice.
     */
    constructor(start, end) {
        if (compareTime(start, end) > 0) {
            throw "Attempted to create a time slice with a start after its end.";
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
     * Returns true if the given time is within this time slice, false otherwise.
     */
    includes(time) {
        return compareTime(this.start, time) <= 0 && compareTime(this.end, time) >= 0;
    }

    /**
     * Formats the TimeSlice as a date range string.
     */
    toDateRangeString() {
        // Start cannot be undefined, as cannot acquire something at an undefined time
        var start = moment.utc(this.start, UNIX_FORMAT, true);
        start = start.format(DATE_STRING_FORMAT);

        var end = this.end;
        if (end === undefined) {
            // TimeSlice has no end
            return start + " and onwards";
        } else if (end === this.start) {
            return start;
        }

        end = moment.utc(end, UNIX_FORMAT, true);
        end = end.format(DATE_STRING_FORMAT);

        return start + " - " + end;
    }
}