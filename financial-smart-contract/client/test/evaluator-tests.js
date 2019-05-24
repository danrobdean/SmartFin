import assert from "assert";

import { dateToUnixTimestamp } from "./../src/js/contract-utils.mjs";
import Evaluator from "./../src/js/evaluator.mjs";
import StepThroughOptions from "./../src/js/step-through-options.mjs";

describe('Evaluator tests', function() {
    const OR_CHOICE_OPTIONS = [true, false];

    var evaluator;

    var dateStringMin;
    var dateStringMax;

    var dateUnixMin;
    var dateUnixMax;

    beforeEach(function() {
        evaluator = new Evaluator();

        dateStringMin = "<01/01/01, 01:23:45>";
        dateStringMax = "<02/02/02, 12:34:56>";

        dateUnixMin = evaluator._dateOrUnixToHorizon(dateStringMin);
        dateUnixMax = evaluator._dateOrUnixToHorizon(dateStringMax);
    });

    it('Converts a date string to a unix timestamp correctly', function() {
        var dateString = "10/11/12, 01:23:45";
        var prettyDateString = "<" + dateString + ">";
        var unix = dateToUnixTimestamp(new Date(dateString));

        assert.equal(evaluator._dateOrUnixToHorizon(prettyDateString), unix);
        assert.equal(evaluator._dateOrUnixToHorizon(unix), unix);
        assert.equal(evaluator._dateOrUnixToHorizon(undefined), undefined);
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

    it('No time slices for a contract not involving truncate', function() {
        evaluator.setContract("anytime give get or one and scale 10 one then zero one");

        assert.deepEqual(evaluator.getTimeSlices().getSlices(), []);
    });

    it('Time slices for truncate are split in the correct place', function() {
        evaluator.setContract("truncate 123 one");

        assert.deepEqual(evaluator.getTimeSlices().getSlices(), [123]);
    });

    it('Time slices for truncate cut off time slices for further-down truncates with later horizons', function() {
        evaluator.setContract("truncate 123 truncate 456 one");

        assert.deepEqual(evaluator.getTimeSlices().getSlices(), [123]);
    });

    it('Time slices for or combine time slices for further-down combinators', function() {
        evaluator.setContract("or truncate 1 one truncate 2 one");

        assert.deepEqual(evaluator.getTimeSlices().getSlices(), [1, 2]);
    });

    it('Time slices for and combine time slices for further-down combinators', function() {
        evaluator.setContract("and truncate 1 one truncate 2 one");

        assert.deepEqual(evaluator.getTimeSlices().getSlices(), [1, 2]);
    });

    it('Time slices for then combine time slices for further-down combinators by merging the second sub-combinator\'s time slices after the first\'s', function() {
        evaluator.setContract("then or truncate 1 one truncate 2 one or truncate 0 one truncate 4 one");

        assert.deepEqual(evaluator.getTimeSlices().getSlices(), [1, 2, 4]);
    });

    it('Time slices for get cut off time slices for truncates with earlier horizons', function() {
        evaluator.setContract("get truncate 127 or truncate 45 one truncate 67 one");

        assert.deepEqual(evaluator.getTimeSlices().getSlices(), [67]);
    });

    it('Anytime time slices are stored in the right order', function() {
        evaluator.setContract("or anytime truncate 1 one anytime truncate 2 one");

        assert.deepEqual(evaluator.getAnytimeTimeSlices()[0].getSlices(), [1]);
        assert.deepEqual(evaluator.getAnytimeTimeSlices()[1].getSlices(), [2]);
    });

    it('Starts step through evaluation by returning the right acquisition time options', function() {
        evaluator.setContract("or anytime truncate 1 one anytime truncate 2 one");

        var options = evaluator.getNextStepThroughOptions();

        assert.equal(options.type, StepThroughOptions.TYPE_ACQUISITION_TIME);
        assert.equal(options.combinatorIndex, -1);
        assert.deepEqual(options.options, evaluator.getTimeSlices().getSlices());
    });

    it('Does not have another step for a contract with no acquisition time-slices or or-choices', function() {
        evaluator.setContract("one");

        assert.equal(evaluator.hasNextStep(), false);
    });

    it('Does not have another step for a contract with one acquisition time-slice and no or-choices', function() {
        evaluator.setContract("truncate 2 one");

        assert.equal(evaluator.hasNextStep(), false);
    });

    it('Returns the correct step-through options for an or-combinator', function() {
        evaluator.setContract("or one zero");

        var options = evaluator.getNextStepThroughOptions();

        assert.equal(options.type, StepThroughOptions.TYPE_OR_CHOICE);
        assert.equal(options.combinatorIndex, 0);
        assert.deepEqual(options.options, OR_CHOICE_OPTIONS);
    });

    it('Returns the correct acquisition-times for an anytime combinator', function() {
        evaluator.setContract("anytime then truncate 1 zero truncate 2 one");

        var options = evaluator.getNextStepThroughOptions().options;
        options.sort((a, b) => a - b);
        evaluator.setStepThroughOption(options[0]);

        options = evaluator.getNextStepThroughOptions();

        assert.equal(options.type, StepThroughOptions.TYPE_ACQUISITION_TIME);
        assert.equal(options.combinatorIndex, 0);
        assert.deepEqual(options.options, [1, 2]);
    });

    it('Returns no acquisition-times for an anytime combinator with no time-slices', function() {
        evaluator.setContract("anytime truncate 2 zero");

        assert.equal(evaluator.hasNextStep(), false);
    });

    it('Returns the correct options for an or-choice within an or-choice\'s first sub-contract', function() {
        evaluator.setContract("or or one zero zero");

        evaluator.setStepThroughOption(true);

        var options = evaluator.getNextStepThroughOptions();

        assert.equal(options.type, StepThroughOptions.TYPE_OR_CHOICE);
        assert.equal(options.combinatorIndex, 1);
        assert.deepEqual(options.options, OR_CHOICE_OPTIONS);
    });

    it('Returns the correct options for an or-choice within an or-choice\'s second sub-contract', function() {
        evaluator.setContract("or one or zero zero");

        evaluator.setStepThroughOption(false);

        var options = evaluator.getNextStepThroughOptions();

        assert.equal(options.type, StepThroughOptions.TYPE_OR_CHOICE);
        assert.equal(options.combinatorIndex, 2);
        assert.deepEqual(options.options, OR_CHOICE_OPTIONS);
    });

    it('Does not return options for an unused or-choice within an or-choice\'s first sub-contract', function() {
        evaluator.setContract("or or one zero zero");

        evaluator.setStepThroughOption(false);

        assert.equal(evaluator.hasNextStep(), false);
    });

    it('Does not return options for an unused or-choice within an or-choice\'s second sub-contract', function() {
        evaluator.setContract("or one or zero zero");

        evaluator.setStepThroughOption(true);

        assert.equal(evaluator.hasNextStep(), false);
    });

    it('Returns correct options for an or-choice as a sub-contract of another combinator', function() {
        evaluator.setContract("scale 5 or zero one");

        var options = evaluator.getNextStepThroughOptions();

        assert.equal(options.type, StepThroughOptions.TYPE_OR_CHOICE);
        assert.equal(options.combinatorIndex, 2);
        assert.deepEqual(options.options, OR_CHOICE_OPTIONS);
    });

    it('Returns correct options for an and combinator', function() {
        evaluator.setContract("and or one zero or zero one");

        assert.equal(evaluator.hasNextStep(), true);

        var options = evaluator.getNextStepThroughOptions();

        assert.equal(options.type, StepThroughOptions.TYPE_OR_CHOICE);
        assert.equal(options.combinatorIndex, 1);
        assert.deepEqual(options.options, OR_CHOICE_OPTIONS);

        evaluator.setStepThroughOption(true);
        var options = evaluator.getNextStepThroughOptions();

        assert.equal(options.type, StepThroughOptions.TYPE_OR_CHOICE);
        assert.equal(options.combinatorIndex, 4);
        assert.deepEqual(options.options, OR_CHOICE_OPTIONS);
    });

    it('Deletes the correct step-through option', function() {
        evaluator.setContract("then truncate 1 anytime then truncate 2 zero truncate 3 or one zero truncate 4 one");

        evaluator.setStepThroughOption(0);

        var options = evaluator.getNextStepThroughOptions();
        evaluator.setStepThroughOption(3);
        evaluator.setStepThroughOption(true);

        evaluator.deleteStepThroughOption(3);

        assert.deepEqual(evaluator.getNextStepThroughOptions(), options);
    });

    it('Evaluates a basic contract correctly', function() {
        evaluator.setContract("one");

        assert.equal(evaluator.evaluate(), "1");

        evaluator.setContract("zero");

        assert.equal(evaluator.evaluate(), "0");
    });

    it('Evaluates a scaled contract correctly', function() {
        evaluator.setContract("scale 5 one");

        assert.equal(evaluator.evaluate(), "5");
    });

    it('Evaluates a contract with observables correctly', function() {
        evaluator.setContract("scale var 0x0 one");

        assert.equal(evaluator.evaluate(), "var * 1");
    });

    it('Evaluates a scaled contract with observables correctly', function() {
        evaluator.setContract("scale var0 0x0 scale 5 scale var1 0x1 scale 10 one");

        assert.equal(evaluator.evaluate(), "var1 * var0 * 50");
    });

    it('Evaluates an and combinator with two scaled/observabled sub-contracts correctly', function() {
        evaluator.setContract("scale var0 0x0 and scale var1 0x0 scale 5 one scale var2 0x0 scale 10 one");

        assert.equal(evaluator.evaluate(), "var0 * (var1 * 5) + (var2 * 10)");
    });

    it('Evaluates a give combinator correctly', function() {
        evaluator.setContract("give one");

        assert.equal(evaluator.evaluate(), "-1");
    });

    it('Evaluates a scaled give combinator correctly', function() {
        evaluator.setContract("scale 5 scale var0 0x0 give scale var1 0x0 scale 10 one");

        assert.equal(evaluator.evaluate(), "var1 * var0 * -50");
    });

    it('Evaluates a give in an and combinator correctly', function() {
        evaluator.setContract("and give scale 5 one one");

        assert.equal(evaluator.evaluate(), "-4");
    });

    it('Evaluates a truncate contract correctly', function() {
        evaluator.setContract("truncate 10 one");

        assert.equal(evaluator.evaluate(), "1");
    });

    it('Evaluates a get contract correctly', function() {
        evaluator.setContract("get truncate 10 then truncate 5 one scale 10 one");

        assert.equal(evaluator.evaluate(), "10");
        evaluator.setContract("get truncate 1 then truncate 5 one scale 10 one");

        assert.equal(evaluator.evaluate(), "1");
    });

    it('Evaluates a then contract correctly', function() {
        evaluator.setContract("then truncate 1 zero truncate 2 one");

        evaluator.setStepThroughOption(0);

        assert.equal(evaluator.evaluate(), "0");

        evaluator.deleteStepThroughOption(-1);
        evaluator.setStepThroughOption(2);

        assert.equal(evaluator.evaluate(), "1");
    });

    it('Evaluates an or contract correctly', function() {
        evaluator.setContract("or one zero");

        evaluator.setStepThroughOption(true);

        assert.equal(evaluator.evaluate(), "1");

        evaluator.deleteStepThroughOption(0);
        evaluator.setStepThroughOption(false);

        assert.equal(evaluator.evaluate(), "0");
    });

    it('Evaluates an anytime contract correctly', function() {
        evaluator.setContract("get truncate 0 anytime then truncate 1 one truncate 2 zero");

        evaluator.setStepThroughOption(0);

        assert.equal(evaluator.evaluate(), "1");

        evaluator.deleteStepThroughOption(3);
        evaluator.setStepThroughOption(2);

        assert.equal(evaluator.evaluate(), "0");
    });
});