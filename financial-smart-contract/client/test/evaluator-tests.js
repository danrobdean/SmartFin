import assert from "assert";

import { dateToUnixTimestamp } from "./../src/js/contract-utils.mjs";
import Evaluator from "./../src/js/evaluator.mjs";

describe('Evaluator tests', function() {
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