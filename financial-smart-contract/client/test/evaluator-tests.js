import assert from "assert";

import { dateToUnixTimestamp } from "./../src/js/contract-utils.mjs";
import Evaluator, { TimeSlices } from "./../src/js/evaluator.mjs";

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