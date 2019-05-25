import React from "react";

import { unixTimestampToDateString } from "./../js/contract-utils.mjs";
import DropDown from "./drop-down.jsx";
import Message from "./message.jsx";
import StepThroughOptions from "./../js/step-through-options.mjs";
import StepThroughValue from "./../js/step-through-value.mjs";
import { dateToUnixTimestamp } from "../js/contract-utils.mjs";

/**
 * Represents the controls for evaluating a contract.
 */
export default class EvaluateControls extends React.Component {
    /**
     * The CSS block name.
     */
    static blockName = "evaluate-controls";

    /**
     * The step's option select.
     */
    optionSelect;

    /**
     * Initialises a new instance of this class.
     * @param props.contract The contract definition string.
     * @param props.evaluator The evaluator instance.
     */
    constructor(props) {
        super(props);

        this.state = {
            options: undefined,
            prevValues: undefined,
            value: undefined,
            contractEvaluationError: "",
            contractEvaluationErrorDetails: ""
        };
    }

    /**
     * Returns the element representing this component.
     */
    render() {
        var messageClassName = EvaluateControls.blockName + "__message";

        var prevValues = null;
        var options = null;
        var noInputMessage = null;
        if (this.props.contract) {
            prevValues = this.renderPrevValues();
            options = this.renderOptions();
        }

        if (!prevValues && !options) {
            noInputMessage = Message.renderInfo("This contract has no value-altering input.");
        }

        return (
            <div className={EvaluateControls.blockName + "__container"}>

                <div className={EvaluateControls.blockName + "__contract-drop-down"}>
                    <DropDown title={"Combinator Contract"} disableChildClick={true}>
                        <div className={EvaluateControls.blockName + "__combinator-contract-container"}>
                            <em>{this.props.contract}</em>
                        </div>
                    </DropDown>
                </div>

                <div className={EvaluateControls.blockName + "__options-container"}>
                    {prevValues}
                    {options}
                    {noInputMessage}
                </div>

                <button
                    className={EvaluateControls.blockName + "__evaluate-button"}
                    onClick={() => this.evaluate()}
                    disabled={this.props.evaluator.hasNextStep()}>
                    Evaluate Contract
                </button>

                {Message.renderError(this.state.contractEvaluationError, this.state.contractEvaluationErrorDetails, messageClassName)}
                {Message.renderSuccess(
                    (this.state.value) ? "Contract Value: " + this.state.value.toString() + " Wei" : "",
                    undefined,
                    messageClassName
                )}
            </div>
        );
    }

    /**
     * Renders the options for the current step's value.
     */
    renderOptions() {
        if (!this.props.evaluator.hasNextStep() || !this.state.options) {
            return null;
        }

        var optionElements = [];
        var options = this.state.options.options;
        var label = this.getLabel(this.state.options.index, this.state.options.type, true);

        for (var i = 0; i < options.length; i++) {
            var option = options[i];

            var text = (this.state.options.type == StepThroughOptions.TYPE_OR_CHOICE)
                ? ((option) ? "First" : "Second") + " sub-contract"
                : unixTimestampToDateString(option);

            if (this.state.options.type == StepThroughOptions.TYPE_ACQUISITION_TIME && i == options.length - 1) {
                text += "...";
            }

            optionElements.push(
                <option key={i} value={option}>{text}</option>
            );
        }

        return (
            <div className={EvaluateControls.blockName + "__option-container"}>
                <span className={EvaluateControls.blockName + "__option-label"}>
                    {label}
                </span>

                <select
                    className={EvaluateControls.blockName + "__options-input"}
                    ref={r => this.optionSelect = r}>
                    {optionElements}
                </select>

                <button
                    className={EvaluateControls.blockName + "__option-button"}
                    onClick={() => this.setOption()}>
                    Set Choice
                </button>
            </div>
        )
    }

    /**
     * Renders the set of previous steps' values.
     */
    renderPrevValues() {
        if (!this.state.prevValues || this.state.prevValues.length == 0) {
            return null;
        }
        var prevValueElements = [];

        for (var i = 0; i < this.state.prevValues.length; i++) {
            var prevValue = this.state.prevValues[i];
            var prevValueElement;
            var label = this.getLabel(prevValue.index, prevValue.type, false);

            if (prevValue.type == StepThroughValue.TYPE_OR_CHOICE) {
                // Or-choice
                prevValueElement = (
                    <div
                        className={EvaluateControls.blockName + "__prev-value-container"}
                        key={i}>
                        <span className={EvaluateControls.blockName + "__prev-value-label"}>
                            {label}
                        </span>

                        <span className={EvaluateControls.blockName + "__prev-value"}>
                            <em>{((prevValue.value) ? "First" : "Second") + " sub-contract"}</em>
                        </span>

                        <button
                            className={EvaluateControls.blockName + "__reset-value-button"}
                            onClick={this.deleteValue.bind(this, prevValue.combinatorIndex)}>
                            Delete
                        </button>
                    </div>
                );
            } else {
                // Acquisition time
                prevValueElement = (
                    <div
                        className={EvaluateControls.blockName + "__prev-value-container"}
                        key={i}>
                        <span className={EvaluateControls.blockName + "__prev-value-label"}>
                            {label}
                        </span>

                        <span className={EvaluateControls.blockName + "__prev-value"}>
                            <em>{unixTimestampToDateString(prevValue.value)}</em>
                        </span>

                        <button
                            className={EvaluateControls.blockName + "__reset-value-button"}
                            onClick={this.deleteValue.bind(this, prevValue.combinatorIndex)}>
                            Delete
                        </button>
                    </div>
                );
            }

            prevValueElements.push(prevValueElement);
        }

        return prevValueElements;
    }

    /**
     * Sets the value for the option of the current step.
     */
    setOption() {
        this.resetError();

        var option = this.optionSelect.value;

        if (this.state.options.type === StepThroughOptions.TYPE_OR_CHOICE) {
            option = option == "true";
        }

        this.props.evaluator.setStepThroughOption(option);

        this.setState({
            options: this.props.evaluator.getNextStepThroughOptions(),
            prevValues: this.props.evaluator.getPrevValues()
        });
    }

    /**
     * Evaluate the contract.
     */
    evaluate() {
        var value = this.props.evaluator.evaluate();

        if (value) {
            this.resetError();

            this.setState({
                value: value
            });
        } else {
            this.setState({
                contractEvaluationError: "Please complete stepping through the contract before evaluating."
            });
        }
    }

    /**
     * Gets the label for a step-through option or value.
     * @param index The option's associated index.
     * @param type The type of step-through option or value.
     */
    getLabel(index, type, isOption) {
        var label;
        var acquisitionTimeType = (isOption) ? StepThroughOptions.TYPE_ACQUISITION_TIME : StepThroughValue.TYPE_ACQUISITION_TIME;
        var anytimeAcquisitionTimeType = (isOption) ? StepThroughOptions.TYPE_ANYTIME_ACQUISITION_TIME : StepThroughValue.TYPE_ANYTIME_ACQUISITION_TIME;
        var orChoiceType = (isOption) ? StepThroughOptions.TYPE_OR_CHOICE : StepThroughValue.TYPE_OR_CHOICE;

        if (type == acquisitionTimeType) {
            label = "Contract Acquisition-Time:";
        } else if (type == anytimeAcquisitionTimeType) {
            label = "Anytime " + index + " Acquisition Time:";
        } else if (type == orChoiceType) {
            label = "Or " + index + " Choice:";
        } else {
            throw "Unexpected option/value type: " + type;
        }

        return label;
    }

    /**
     * Resets the contract.
     */
    resetContract() {
        this.props.evaluator.setContract(this.props.contract);

        this.resetError();
        this.setState({
            value: undefined,
            contractEvaluationError: "",
            contractEvaluationErrorDetails: "",
            options: undefined,
            prevValues: undefined
        })

        if (this.props.evaluator.hasNextStep()) {
            this.setState({
                options: this.props.evaluator.getNextStepThroughOptions(),
                prevValues: this.props.evaluator.getPrevValues()
            });
        }
    }


    /**
     * Delete the value chosen for a step-through option at the given combinator index, and all following chosen values.
     * @param combinatorIndex The combinator index of the chosen value to delete.
     */
    deleteValue(combinatorIndex) {
        this.props.evaluator.deleteStepThroughOption(combinatorIndex);

        this.setState({
            options: this.props.evaluator.getNextStepThroughOptions(),
            prevValues: this.props.evaluator.getPrevValues()
        });
    }

    /**
     * Reset the error state.
     */
    resetError() {
        this.setState({
            contractEvaluationError: "",
            contractEvaluationErrorDetails: ""
        });
    }
}