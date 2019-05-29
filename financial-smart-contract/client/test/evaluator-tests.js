import assert from "assert";
import moment from "moment";

import { UNIX_FORMAT, DATE_STRING_FORMAT } from "./../src/js/contract-utils.mjs";
import Evaluator from "./../src/js/evaluator.mjs";
import StepThroughOptions from "./../src/js/step-through-options.mjs";
import TimeSlice from "../src/js/time-slice.mjs";

describe('Evaluator tests', function() {
    const OR_CHOICE_OPTIONS = [true, false];

    var evaluator;

    var dateStringMin;
    var dateStringMax;

    var dateUnixMin;
    var dateUnixMid0;
    var dateUnixMid1;
    var dateUnixMax;

    beforeEach(function() {
        evaluator = new Evaluator();

        var nowUnix = moment().unix();

        dateUnixMin = nowUnix + 10000;
        dateUnixMax = nowUnix + 100000;
        dateUnixMid0 = nowUnix + 20000;
        dateUnixMid1 = nowUnix + 30000;

        dateStringMin = "<" + moment.utc(dateUnixMin, UNIX_FORMAT, true).format(DATE_STRING_FORMAT) + ">";
        dateStringMax = "<" + moment.utc(dateUnixMax, UNIX_FORMAT, true).format(DATE_STRING_FORMAT) + ">";
    });

    it('Converts a date string to a unix timestamp correctly', function() {
        var dateString = "10/11/2012 01:23:45 +00";
        var prettyDateString = "<" + dateString + ">";
        var unix = moment.utc(dateString, DATE_STRING_FORMAT, true).unix();

        assert.equal(evaluator._dateToUnix(prettyDateString), unix);
    });

    it('Calculates the horizon of zero correctly', function() {
        evaluator.setContract("zero");
        assert.equal(evaluator.getHorizon(), undefined);
    });

    it('Calculates the horizon of one correctly', function() {
        evaluator.setContract("one");
        assert.equal(evaluator.getHorizon(), undefined);
    });

    it('Calculates the horizon of truncate correctly', function() {
        // Deserialized dates
        evaluator.setContract("truncate " + dateStringMin + " one");
        assert.equal(evaluator.getHorizon(), dateUnixMin);

        evaluator.setContract("truncate " + dateStringMax + " truncate " + dateStringMin + " one");
        assert.equal(evaluator.getHorizon(), dateUnixMin);

        evaluator.setContract("truncate " + dateStringMin + " truncate " + dateStringMax + " one")
        assert.equal(evaluator.getHorizon(), dateUnixMin);

        // Pre-serialized dates
        evaluator.setContract("truncate " + dateUnixMin + " one");
        assert.equal(evaluator.getHorizon(), dateUnixMin);

        evaluator.setContract("truncate " + dateUnixMax + " truncate " + dateUnixMin + " one");
        assert.equal(evaluator.getHorizon(), dateUnixMin);

        evaluator.setContract("truncate " + dateUnixMin + " truncate " + dateUnixMax + " one");
        assert.equal(evaluator.getHorizon(), dateUnixMin);
    });

    it('Calculates the horizon of give correctly', function() {
        evaluator.setContract("give one");
        assert.equal(evaluator.getHorizon(), undefined);

        evaluator.setContract("give truncate " + dateStringMin + " one");
        assert.equal(evaluator.getHorizon(), dateUnixMin);
    });

    it('Calculates the horizon of get correctly', function() {
        evaluator.setContract("get one");
        assert.equal(evaluator.getHorizon(), undefined);

        evaluator.setContract("get truncate " + dateStringMin + " one");
        assert.equal(evaluator.getHorizon(), dateUnixMin);
    });

    it('Calculates the horizon of anytime correctly', function() {
        evaluator.setContract("anytime one");
        assert.equal(evaluator.getHorizon(), undefined);

        evaluator.setContract("anytime truncate " + dateStringMin + " one");
        assert.equal(evaluator.getHorizon(), dateUnixMin);
    });

    it('Calculates the horizon of scale correctly', function() {
        evaluator.setContract("scale 10 one");
        assert.equal(evaluator.getHorizon(), undefined);

        evaluator.setContract("scale 10 truncate " + dateStringMin + " one");
        assert.equal(evaluator.getHorizon(), dateUnixMin);

        // Observables from deserialized definition
        evaluator.setContract("scale name <0x1> one");
        assert.equal(evaluator.getHorizon(), undefined);

        evaluator.setContract("scale name <0x1> truncate " + dateStringMin + " one");
        assert.equal(evaluator.getHorizon(), dateUnixMin);

        // Observables from pre-serialized definition
        evaluator.setContract("scale name 0x1 one");
        assert.equal(evaluator.getHorizon(), undefined);

        evaluator.setContract("scale name 0x1 truncate " + dateStringMin + " one");
        assert.equal(evaluator.getHorizon(), dateUnixMin);
    });

    it('Calculates the horizon of and correctly', function() {
        evaluator.setContract("and one one");
        assert.equal(evaluator.getHorizon(), undefined);

        evaluator.setContract("and truncate " + dateStringMin + " one one");
        assert.equal(evaluator.getHorizon(), undefined);

        evaluator.setContract("and truncate " + dateStringMin + " one truncate " + dateStringMax + " one");
        assert.equal(evaluator.getHorizon(), dateUnixMax);

        evaluator.setContract("and truncate " + dateStringMax + " one truncate " + dateStringMin + " one");
        assert.equal(evaluator.getHorizon(), dateUnixMax);

        evaluator.setContract("and one truncate " + dateStringMin + " one");
        assert.equal(evaluator.getHorizon(), undefined);
    });

    it('Calculates the horizon of or correctly', function() {
        evaluator.setContract("or one one");
        assert.equal(evaluator.getHorizon(), undefined);

        evaluator.setContract("or truncate " + dateStringMin + " one one");
        assert.equal(evaluator.getHorizon(), undefined);

        evaluator.setContract("or truncate " + dateStringMin + " one truncate " + dateStringMax + " one");
        assert.equal(evaluator.getHorizon(), dateUnixMax);

        evaluator.setContract("or truncate " + dateStringMax + " one truncate " + dateStringMin + " one");
        assert.equal(evaluator.getHorizon(), dateUnixMax);

        evaluator.setContract("or one truncate " + dateStringMin + " one");
        assert.equal(evaluator.getHorizon(), undefined);
    });

    it('Calculates the horizon of then correctly', function() {
        evaluator.setContract("then one one");
        assert.equal(evaluator.getHorizon(), undefined);

        evaluator.setContract("then truncate " + dateStringMin + " one one");
        assert.equal(evaluator.getHorizon(), undefined);

        evaluator.setContract("then truncate " + dateStringMin + " one truncate " + dateStringMax + " one");
        assert.equal(evaluator.getHorizon(), dateUnixMax);

        evaluator.setContract("then truncate " + dateStringMax + " one truncate " + dateStringMin + " one");
        assert.equal(evaluator.getHorizon(), dateUnixMax);

        evaluator.setContract("then one truncate " + dateStringMin + " one");
        assert.equal(evaluator.getHorizon(), undefined);
    });

    it('Modifying time slices doesn\'t modify base object', function() {
        evaluator.setContract("one");
        var timeSlices = evaluator.getTimeSlices();
        timeSlices.split(1);

        assert.notEqual(timeSlices, evaluator.getTimeSlices());
    });

    it('No time slices (besides infinite horizon) for a contract not involving truncate', function() {
        evaluator.setContract("anytime give get or one and scale 10 one then zero one");

        assert.deepEqual(evaluator.getTimeSlices().getSlices(), [new TimeSlice(0, undefined)]);
    });

    it('Time slices for truncate are split in the correct place', function() {
        evaluator.setContract("truncate 123 one");

        assert.deepEqual(evaluator.getTimeSlices().getSlices(), [new TimeSlice(0, 123)]);
    });

    it('Time slices for truncate cut off time slices for further-down truncates with later horizons', function() {
        evaluator.setContract("truncate 123 truncate 456 one");

        assert.deepEqual(evaluator.getTimeSlices().getSlices(), [new TimeSlice(0, 123)]);
    });

    it('Time slices for or combine time slices for further-down combinators', function() {
        evaluator.setContract("or truncate 1 one truncate 2 one");

        assert.deepEqual(evaluator.getTimeSlices().getSlices(), [new TimeSlice(0, 1), new TimeSlice(2, 2)]);
    });

    it('Time slices for and combine time slices for further-down combinators', function() {
        evaluator.setContract("and truncate 1 one truncate 2 one");

        assert.deepEqual(evaluator.getTimeSlices().getSlices(), [new TimeSlice(0, 1), new TimeSlice(2, 2)]);
    });

    it('Time slices for then combine time slices for further-down combinators by merging the second sub-combinator\'s time slices after the first\'s', function() {
        evaluator.setContract("then or truncate 1 one truncate 2 one or truncate 0 one truncate 4 one");

        assert.deepEqual(evaluator.getTimeSlices().getSlices(), [new TimeSlice(0, 1), new TimeSlice(2, 2), new TimeSlice(3, 4)]);
    });

    it('Time slices for get cut off time slices for truncates with earlier horizons', function() {
        evaluator.setContract("get truncate 127 or truncate 45 one truncate 67 one");

        assert.deepEqual(evaluator.getTimeSlices().getSlices(), [new TimeSlice(0, 67)]);
    });

    it('Anytime time slices are stored in the right order', function() {
        evaluator.setContract("or anytime truncate 1 one anytime truncate 2 one");

        assert.deepEqual(evaluator.getAnytimeTimeSlices()[0].getSlices(), [new TimeSlice(0, 1)]);
        assert.deepEqual(evaluator.getAnytimeTimeSlices()[1].getSlices(), [new TimeSlice(0, 2)]);
    });

    it('Starts step through evaluation by returning the right acquisition time options for a simple contract', function() {
        evaluator.setContract("one");

        var options = evaluator.getNextStepThroughOptions(false);
        var expectedOptions = [
            // The first range starts at the current time, just copy it to prevent sync issues
            new TimeSlice(options.options[0].getStart(), undefined)
        ];

        assert.equal(options.type, StepThroughOptions.TYPE_ACQUISITION_TIME);
        assert.equal(options.combinatorIndex, -1);
        assert.deepEqual(options.options, expectedOptions);
    });

    it('Starts step through evaluation by returning the right acquisition time options', function() {
        evaluator.setContract("or anytime truncate " + dateUnixMin + " one anytime truncate " + dateUnixMax + " one");

        var options = evaluator.getNextStepThroughOptions(false);
        var expectedOptions = [
            // The first range starts at the current time, just copy it to prevent sync issues
            new TimeSlice(options.options[0].getStart(), dateUnixMin),
            new TimeSlice(dateUnixMin + 1, dateUnixMax),
            new TimeSlice(dateUnixMax + 1, undefined)
        ];

        assert.equal(options.type, StepThroughOptions.TYPE_ACQUISITION_TIME);
        assert.equal(options.combinatorIndex, -1);
        assert.deepEqual(options.options, expectedOptions);
    });

    it('Does not have another step for a contract with no acquisition time-slices or or-choices after acquisition', function() {
        evaluator.setContract("one");

        evaluator.setStepThroughOption(0);
        assert.equal(evaluator.hasNextStep(), false);
    });

    it('Has two options for a contract with one acquisition time-slice and no or-choices', function() {
        evaluator.setContract("truncate " + dateUnixMin + " one");

        var options = evaluator.getNextStepThroughOptions(false).options;
        var expectedOptions = [
            new TimeSlice(options[0].getStart(), dateUnixMin),
            new TimeSlice(dateUnixMin + 1, undefined)
        ];

        assert.deepEqual(options, expectedOptions);
    });

    it('Returns the correct step-through options for an or-combinator', function() {
        evaluator.setContract("or one zero");

        var options = evaluator.getNextStepThroughOptions(false);
        evaluator.setStepThroughOption(options.options[0]);
        options = evaluator.getNextStepThroughOptions(false);

        assert.equal(options.type, StepThroughOptions.TYPE_OR_CHOICE);
        assert.equal(options.combinatorIndex, 0);
        assert.deepEqual(options.options, OR_CHOICE_OPTIONS);
    });

    it('Returns the correct acquisition-times for an anytime combinator', function() {
        evaluator.setContract("anytime then truncate " + dateUnixMin + " zero truncate " + dateUnixMax + " one");

        var options = evaluator.getNextStepThroughOptions(false);
        evaluator.setStepThroughOption(options.options[0]);
        options = evaluator.getNextStepThroughOptions(false);

        assert.equal(options.type, StepThroughOptions.TYPE_ANYTIME_ACQUISITION_TIME);
        assert.equal(options.combinatorIndex, 0);
        assert.deepEqual(options.options, [new TimeSlice(options.options[0].getStart(), dateUnixMin), new TimeSlice(dateUnixMin + 1, dateUnixMax)]);
    });

    it('Returns no acquisition-times for an anytime combinator with no time-slices after acquisition', function() {
        evaluator.setContract("anytime truncate " + dateUnixMin + " zero");

        var options = evaluator.getNextStepThroughOptions(false);
        evaluator.setStepThroughOption(options.options[0]);
        options = evaluator.getNextStepThroughOptions(false);

        assert.equal(evaluator.hasNextStep(), false);
    });

    it('Returns the correct options for an or-choice within an or-choice\'s first sub-contract', function() {
        evaluator.setContract("or or one zero zero");

        var options = evaluator.getNextStepThroughOptions(false);
        evaluator.setStepThroughOption(options.options[0]);
        evaluator.setStepThroughOption(true);

        options = evaluator.getNextStepThroughOptions(false);

        assert.equal(options.type, StepThroughOptions.TYPE_OR_CHOICE);
        assert.equal(options.combinatorIndex, 1);
        assert.deepEqual(options.options, OR_CHOICE_OPTIONS);
    });

    it('Returns the correct options for an or-choice within an or-choice\'s second sub-contract', function() {
        evaluator.setContract("or one or zero zero");


        var options = evaluator.getNextStepThroughOptions(false);
        evaluator.setStepThroughOption(options.options[0]);
        evaluator.setStepThroughOption(false);

        options = evaluator.getNextStepThroughOptions(false);

        assert.equal(options.type, StepThroughOptions.TYPE_OR_CHOICE);
        assert.equal(options.combinatorIndex, 2);
        assert.deepEqual(options.options, OR_CHOICE_OPTIONS);
    });

    it('Does not return options for an unused or-choice within an or-choice\'s first sub-contract', function() {
        evaluator.setContract("or or one zero zero");

        var options = evaluator.getNextStepThroughOptions(false);
        evaluator.setStepThroughOption(options.options[0]);
        evaluator.setStepThroughOption(false);

        assert.equal(evaluator.hasNextStep(), false);
    });

    it('Does not return options for an unused or-choice within an or-choice\'s second sub-contract', function() {
        evaluator.setContract("or one or zero zero");

        var options = evaluator.getNextStepThroughOptions(false);
        evaluator.setStepThroughOption(options.options[0]);
        evaluator.setStepThroughOption(true);

        assert.equal(evaluator.hasNextStep(), false);
    });

    it('Returns correct options for an or-choice as a sub-contract of another combinator', function() {
        evaluator.setContract("scale 5 or zero one");

        var options = evaluator.getNextStepThroughOptions(false);
        evaluator.setStepThroughOption(options.options[0]);
        options = evaluator.getNextStepThroughOptions(false);

        assert.equal(options.type, StepThroughOptions.TYPE_OR_CHOICE);
        assert.equal(options.combinatorIndex, 2);
        assert.deepEqual(options.options, OR_CHOICE_OPTIONS);
    });

    it('Returns correct options for an and combinator', function() {
        evaluator.setContract("and or one zero or zero one");

        var options = evaluator.getNextStepThroughOptions(false);
        evaluator.setStepThroughOption(options.options[0]);

        assert.equal(evaluator.hasNextStep(), true);

        options = evaluator.getNextStepThroughOptions(false);

        assert.equal(options.type, StepThroughOptions.TYPE_OR_CHOICE);
        assert.equal(options.combinatorIndex, 1);
        assert.deepEqual(options.options, OR_CHOICE_OPTIONS);

        evaluator.setStepThroughOption(true);
        options = evaluator.getNextStepThroughOptions(false);

        assert.equal(options.type, StepThroughOptions.TYPE_OR_CHOICE);
        assert.equal(options.combinatorIndex, 4);
        assert.deepEqual(options.options, OR_CHOICE_OPTIONS);
    });

    it('Deletes the correct step-through option', function() {
        evaluator.setContract("then truncate " + dateUnixMin + " anytime then truncate " + dateUnixMid0 + " zero truncate "
            + dateUnixMid1 + " or one zero truncate " + dateUnixMax + " one");

        var options = evaluator.getNextStepThroughOptions(false);
        evaluator.setStepThroughOption(options.options[0]);

        options = evaluator.getNextStepThroughOptions(false);
        evaluator.setStepThroughOption(options.options[1]);

        evaluator.setStepThroughOption(true);

        evaluator.deleteStepThroughOption(3);

        assert.deepEqual(evaluator.getNextStepThroughOptions(false), options);
    });

    it('Evaluates a basic contract correctly', function() {
        evaluator.setContract("one");

        var options = evaluator.getNextStepThroughOptions(false);
        evaluator.setStepThroughOption(options.options[0]);

        assert.equal(evaluator.evaluate(false), "1 Wei");

        evaluator.setContract("zero");

        options = evaluator.getNextStepThroughOptions(false);
        evaluator.setStepThroughOption(options.options[0]);

        assert.equal(evaluator.evaluate(false), "0 Wei");
    });

    it('Evaluates a scaled contract correctly', function() {
        evaluator.setContract("scale 5 one");

        var options = evaluator.getNextStepThroughOptions(false);
        evaluator.setStepThroughOption(options.options[0]);

        assert.equal(evaluator.evaluate(false), "5 Wei");
    });

    it('Evaluates a contract with observables correctly', function() {
        evaluator.setContract("scale var 0x0 one");

        var options = evaluator.getNextStepThroughOptions(false);
        evaluator.setStepThroughOption(options.options[0]);

        assert.equal(evaluator.evaluate(false), "var * 1 Wei");
    });

    it('Evaluates a scaled contract with observables correctly', function() {
        evaluator.setContract("scale var0 0x0 scale 5 scale var1 0x1 scale 10 one");

        var options = evaluator.getNextStepThroughOptions(false);
        evaluator.setStepThroughOption(options.options[0]);

        assert.equal(evaluator.evaluate(false), "var0 * var1 * 50 Wei");
    });

    it('Evaluates an and combinator with two scaled/observabled sub-contracts correctly', function() {
        evaluator.setContract("scale var0 0x0 and scale var1 0x0 scale 5 one scale var2 0x0 scale 10 one");

        var options = evaluator.getNextStepThroughOptions(false);
        evaluator.setStepThroughOption(options.options[0]);

        assert.equal(evaluator.evaluate(false), "var0 * (var1 * 5 Wei + var2 * 10 Wei)");
    });

    it('Evaluates a give combinator correctly', function() {
        evaluator.setContract("give one");

        var options = evaluator.getNextStepThroughOptions(false);
        evaluator.setStepThroughOption(options.options[0]);

        assert.equal(evaluator.evaluate(false), "-1 Wei");
    });

    it('Evaluates a scaled give combinator correctly', function() {
        evaluator.setContract("scale 5 scale var0 0x0 give scale var1 0x0 scale 10 one");

        var options = evaluator.getNextStepThroughOptions(false);
        evaluator.setStepThroughOption(options.options[0]);

        assert.equal(evaluator.evaluate(false), "var0 * var1 * -50 Wei");
    });

    it('Evaluates a give in an and combinator correctly', function() {
        evaluator.setContract("and give scale 5 one one");

        var options = evaluator.getNextStepThroughOptions(false);
        evaluator.setStepThroughOption(options.options[0]);

        assert.equal(evaluator.evaluate(false), "(-5 Wei + 1 Wei)");
    });

    it('Evaluates a truncate contract correctly', function() {
        evaluator.setContract("truncate " + dateUnixMin + " one");

        var options = evaluator.getNextStepThroughOptions(false);
        evaluator.setStepThroughOption(options.options[0]);

        assert.equal(evaluator.evaluate(false), "1 Wei");
    });

    it('Evaluates a get contract correctly', function() {
        evaluator.setContract("get truncate " + dateUnixMax + " then truncate " + dateUnixMin + " one scale 10 one");

        var options = evaluator.getNextStepThroughOptions(false);
        evaluator.setStepThroughOption(options.options[0]);

        assert.equal(evaluator.evaluate(false), "10 Wei");

        evaluator.setContract("get truncate " + dateUnixMin + " then truncate " + dateUnixMax + " one scale 10 one");

        var options = evaluator.getNextStepThroughOptions(false);
        evaluator.setStepThroughOption(options.options[0]);

        assert.equal(evaluator.evaluate(false), "1 Wei");
    });

    it('Evaluates a then contract correctly', function() {
        evaluator.setContract("then truncate " + dateUnixMin + " zero truncate " + dateUnixMax + " one");

        var options = evaluator.getNextStepThroughOptions(false);
        evaluator.setStepThroughOption(options.options[0]);

        assert.equal(evaluator.evaluate(false), "0 Wei");

        evaluator.deleteStepThroughOption(-1);

        evaluator.setStepThroughOption(options.options[1]);

        assert.equal(evaluator.evaluate(false), "1 Wei");
    });

    it('Evaluates an or contract correctly', function() {
        evaluator.setContract("or one zero");

        var options = evaluator.getNextStepThroughOptions(false);
        evaluator.setStepThroughOption(options.options[0]);
        evaluator.setStepThroughOption(true);

        assert.equal(evaluator.evaluate(false), "1 Wei");

        evaluator.deleteStepThroughOption(0);
        evaluator.setStepThroughOption(false);

        assert.equal(evaluator.evaluate(false), "0 Wei");
    });

    it('Evaluates an anytime contract correctly', function() {
        evaluator.setContract("get truncate " + dateUnixMin + " anytime then truncate " + dateUnixMid0 + " one truncate " + dateUnixMid1 + " zero");

        var options = evaluator.getNextStepThroughOptions(false);
        evaluator.setStepThroughOption(options.options[0]);

        options = evaluator.getNextStepThroughOptions(false);
        evaluator.setStepThroughOption(options.options[1]);

        assert.equal(evaluator.evaluate(false), "0 Wei");

        evaluator.deleteStepThroughOption(3);

        options = evaluator.getNextStepThroughOptions(false);
        evaluator.setStepThroughOption(options.options[0]);

        assert.equal(evaluator.evaluate(false), "1 Wei");
    });

    it('Evaluates a simple contract with acquisition times showing correctly', function() {
        evaluator.setContract("one");

        var options = evaluator.getNextStepThroughOptions(false);
        evaluator.setStepThroughOption(options.options[0]);

        assert.equal(evaluator.evaluate(true), "1 Wei <" + options.options[0].toDateRangeString() + ">");
    });

    it('Evaluates a contract scaled by an observable with acquisition times showing correctly', function() {
        evaluator.setContract("scale var1 0x0 one");

        var options = evaluator.getNextStepThroughOptions(false);
        evaluator.setStepThroughOption(options.options[0]);

        assert.equal(evaluator.evaluate(true), "var1 <" + options.options[0].toDateRangeString() + "> * 1 Wei <" + options.options[0].toDateRangeString() + ">");
    });

    it('Evaluates a contract scaled by an observable with a get sub-contract with acquisition times showing correctly', function() {
        evaluator.setContract("scale var1 0x0 get truncate " + dateUnixMin + " one");

        var options = evaluator.getNextStepThroughOptions(false);
        evaluator.setStepThroughOption(options.options[0]);

        assert.equal(evaluator.evaluate(true), "var1 <" + options.options[0].toDateRangeString() + "> * 1 Wei <" + new TimeSlice(dateUnixMin, dateUnixMin).toDateRangeString() + ">");
    });
});