import assert from "assert";

import { dateToUnixTimestamp } from "./../src/js/contract-utils.mjs";
import Evaluator from "./../src/js/evaluator.mjs";
import StepThroughOptions from "./../src/js/step-through-options.mjs";
import TimeSlices from "./../src/js/time-slices.mjs";

describe.only('Evaluator tests', function() {
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

    it("Calculates the horizon of give correctly", function() {
        evaluator.setContract("give one");
        assert.equal(evaluator.getHorizon(), undefined);

        evaluator.setContract("give truncate " + dateStringMin + " one");
        assert.equal(evaluator.getHorizon(), dateUnixMin);
    });

    it("Calculates the horizon of get correctly", function() {
        evaluator.setContract("get one");
        assert.equal(evaluator.getHorizon(), undefined);

        evaluator.setContract("get truncate " + dateStringMin + " one");
        assert.equal(evaluator.getHorizon(), dateUnixMin);
    });

    it("Calculates the horizon of anytime correctly", function() {
        evaluator.setContract("anytime one");
        assert.equal(evaluator.getHorizon(), undefined);

        evaluator.setContract("anytime truncate " + dateStringMin + " one");
        assert.equal(evaluator.getHorizon(), dateUnixMin);
    });

    it("Calculates the horizon of scale correctly", function() {
        evaluator.setContract("scale 10 one");
        assert.equal(evaluator.getHorizon(), undefined);

        evaluator.setContract("scale 10 truncate " + dateStringMin + " one");
        assert.equal(evaluator.getHorizon(), dateUnixMin);

        // Observables from deserialized definition
        evaluator.setContract("scale obs <0x1> one");
        assert.equal(evaluator.getHorizon(), undefined);

        evaluator.setContract("scale obs <0x1> truncate " + dateStringMin + " one");
        assert.equal(evaluator.getHorizon(), dateUnixMin);

        // Observables from pre-serialized definition
        evaluator.setContract("scale obs 0x1 one");
        assert.equal(evaluator.getHorizon(), undefined);

        evaluator.setContract("scale obs 0x1 truncate " + dateStringMin + " one");
        assert.equal(evaluator.getHorizon(), dateUnixMin);
    });

    it("Calculates the horizon of and correctly", function() {
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

    it("Calculates the horizon of or correctly", function() {
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

    it("Calculates the horizon of then correctly", function() {
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

    it("Modifying time slices doesn't modify base object", function() {
        evaluator.setContract("one");
        var timeSlices = evaluator.getTimeSlices();
        timeSlices.split(1);

        assert.notEqual(timeSlices, evaluator.getTimeSlices());
    });

    it("No time slices for a contract not involving truncate", function() {
        evaluator.setContract("anytime give get or one and scale 10 one then zero one");

        assert.deepEqual(evaluator.getTimeSlices().getSlices(), []);
    });

    it("Time slices for truncate are split in the correct place", function() {
        evaluator.setContract("truncate 123 one");

        assert.deepEqual(evaluator.getTimeSlices().getSlices(), [123]);
    });

    it("Time slices for truncate cut off time slices for further-down truncates with later horizons", function() {
        evaluator.setContract("truncate 123 truncate 456 one");

        assert.deepEqual(evaluator.getTimeSlices().getSlices(), [123]);
    });

    it("Time slices for or combine time slices for further-down combinators", function() {
        evaluator.setContract("or truncate 1 one truncate 2 one");

        assert.deepEqual(evaluator.getTimeSlices().getSlices(), [1, 2]);
    });

    it("Time slices for and combine time slices for further-down combinators", function() {
        evaluator.setContract("and truncate 1 one truncate 2 one");

        assert.deepEqual(evaluator.getTimeSlices().getSlices(), [1, 2]);
    });

    it("Time slices for then combine time slices for further-down combinators by merging the second sub-combinator's time slices after the first's", function() {
        evaluator.setContract("then or truncate 1 one truncate 2 one or truncate 0 one truncate 4 one");

        assert.deepEqual(evaluator.getTimeSlices().getSlices(), [1, 2, 4]);
    });

    it("Time slices for get cut off time slices for truncates with earlier horizons", function() {
        evaluator.setContract("get truncate 127 or truncate 45 one truncate 67 one");

        assert.deepEqual(evaluator.getTimeSlices().getSlices(), [67]);
    });

    it("Anytime time slices are stored in the right order", function() {
        evaluator.setContract("or anytime truncate 1 one anytime truncate 2 one");

        assert.deepEqual(evaluator.getAnytimeTimeSlices()[0].getSlices(), [1]);
        assert.deepEqual(evaluator.getAnytimeTimeSlices()[1].getSlices(), [2]);
    });

    it("Starts step through evaluation by returning the right acquisition time options", function() {
        evaluator.setContract("or anytime truncate 1 one anytime truncate 2 one");

        var options = evaluator.startStepThroughEvaluation();

        assert.equal(options.getType(), StepThroughOptions.TYPE_ACQUISITION_TIME);
        assert.deepEqual(options.getOptions(), evaluator.getTimeSlices().getSlices());
    });
});

describe("TimeSlices tests", function() {
    var timeSlices;

    beforeEach(function() {
        timeSlices = new TimeSlices();
    });

    it("Has no slices initially", function() {
        assert.deepEqual(timeSlices.getSlices(), []);
    });

    it("Splits time slices correctly", function() {
        timeSlices.split(1);
        timeSlices.split(3);
        timeSlices.split(2);

        assert.deepEqual(timeSlices.getSlices(), [1, 2, 3]);
    });

    it("Cuts off time slices correctly", function() {
        timeSlices.split(1);
        timeSlices.split(3);
        timeSlices.cut(2);

        assert.deepEqual(timeSlices.getSlices(), [1, 2]);
    });

    it("Merges time slices correctly", function() {
        var otherTimeSlices = new TimeSlices();

        timeSlices.split(1);
        timeSlices.split(3);
        timeSlices.split(5);

        otherTimeSlices.split(2);
        otherTimeSlices.split(4);
        otherTimeSlices.split(6);

        timeSlices.merge(otherTimeSlices);

        assert.deepEqual(timeSlices.getSlices(), [1, 2, 3, 4, 5, 6]);
    });

    it("Merge after merges time slices correctly", function() {
        var otherTimeSlices = new TimeSlices();

        timeSlices.split(1);
        timeSlices.split(3);
        timeSlices.split(5);

        otherTimeSlices.split(2);
        otherTimeSlices.split(4);
        otherTimeSlices.split(6);
        otherTimeSlices.split(7);

        timeSlices.mergeAfter(otherTimeSlices);

        assert.deepEqual(timeSlices.getSlices(), [1, 3, 5, 6, 7]);
    });
});