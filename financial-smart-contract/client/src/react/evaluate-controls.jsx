import React from "react";

import { unixTimestampToDateString } from "./../js/contract-utils.mjs";
import Message from "./message.jsx";
import StepThroughOptions from "./../js/step-through-options.mjs";
import StepThroughValue from "./../js/step-through-value.mjs";

/**
 * Represents the controls for evaluating a contract.
 */
export default class EvaluateControls extends React.Component {
    /**
     * The CSS block name.
     */
    static blockName = "evaluate-controls";

    /**
     * The step's option input.
     */
    optionInput;

    /**
     * Initialises a new instance of this class.
     * @param props.contractDefinition The contract definition string.
     * @param props.evaluator The evaluator instance.
     */
    constructor(props) {
        super(props);

        this.state = {
            options: undefined,
            value: undefined,
            chosenOption: undefined,
            contractEvaluationError: "",
            contractEvaluationErrorDetails: ""
        };
    }

    /**
     * Returns the element representing this component.
     */
    render() {
        var messageClassName = EvaluateControls.blockName + "__message";

        return (
            <div className={EvaluateControls.blockName + "__container"}>
                <div className={EvaluateControls.blockName + "__options-container"}>
                    {this.renderPrevValues()}
                    {this.renderOptions()}
                </div>

                <button
                    className={EvaluateControls.blockName + "__evaluate_button"}
                    onClick={() => this.evaluate()}
                    disabled={!this.props.evaluator.hasNextStep()}>
                    Evaluate Contract
                </button>

                {Message.renderError(this.state.contractEvaluationError, this.state.contractEvaluationErrorDetails, messageClassName)}
                {Message.renderSuccess(
                    (this.state.value) ? "Contract Value: " + value.toString() : "",
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
        if (!this.props.evaluator.hasNextStep()) {
            return null;
        }

        var optionElements = [];

        for (var i = 0; i < this.state.options; i++) {
            var option = options[i];
            var text = (option.type == StepThroughOptions.TYPE_ACQUISITION_TYPE)
                ? unixTimestampToDateString(option.value)
                : ((option.value) ? "First" : "Second") + " sub-contract";

            optionElements.push(
                <option key={i} value={options[i]}>{text}</option>
            );
        }

        return (
            <div className={EvaluateControls.blockName + "__options-container"}>
                <input
                    className={EvaluateControls.blockName + "__options-input"}
                    ref={r => this.optionInput = r}
                    onChange={e => this.handleOptionInputChange()}>
                    {optionElements}
                </input>

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
        var prevValues = this.props.evaluator.getPrevValues();

        if (prevValues && prevValues.length > 0) {
            var prevValueElements = [];

            for (var i = 0; i < prevValues.length; i++) {
                var prevValue = prevValues[i];
                var prevValueElement;

                if (prevValue.type == StepThroughValue.TYPE_ACQUISITION_TYPE) {
                    // Acquisition time
                    prevValue = (
                        <div
                            className={EvaluateControls.blockName + "__prev-value-container"}
                            key={i}>
                            <span className={EvaluateControls.blockName + "__prev-value"}>
                                {unixTimestampToDateString(prevValue.value)}
                            </span>
                            <button
                                className={EvaluateControls.blockName + "__reset-value-button"}
                                onClick={() => this.deleteValue(prevValue.combinatorIndex)}>
                                Delete
                            </button>
                        </div>
                    );
                } else {
                    // Or-choice
                    prevValue = (
                        <div
                            className={EvaluateControls.blockName + "__prev-value-container"}
                            key={i}>
                            <span className={EvaluateControls.blockName + "__prev-value"}>
                                {((prevValue.value) ? "First" : "Second") + " sub-contract"}
                            </span>
                            <button
                                className={EvaluateControls.blockName + "__reset-value-button"}
                                onClick={() => this.deleteValue(prevValue.combinatorIndex)}>
                                Delete
                            </button>
                        </div>
                    );
                }

                prevValueElements.push(prevValueElement);
            }

            return prevValueElements;
        }
    }

    /**
     * Called when the component mounts on the DOM.
     */
    componentDidMount() {
        if (this.props.contractDefinition) {
            this.resetContract();
        }
    }

    /**
     * Called when the component's props or state update.
     * @param prevProps The previous component properties.
     */
    componentDidUpdate(prevProps) {
        // If contract definition changes, restart evaluation
        if (prevProps.contractDefinition !== this.props.contractDefinition) {
            this.resetContract();
        }
    }

    /**
     * Sets the value for the option of the current step.
     */
    setOption() {
        this.props.evaluator.setStepThroughOption(this.state.chosenOption);

        this.setState({
            options: this.props.evaluator.getNextStepThroughOptions()
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
     * Resets the contract.
     */
    resetContract() {
        this.props.evaluator.setContract(this.props.contractDefinition);

        this.resetError();

        if (this.props.evaluator.hasNextStep()) {
            this.setState({
                options: this.props.evaluator.getNextStepThroughOptions()
            });
        }
    }


    /**
     * Delete the value chosen for a step-through option at the given combinator index, and all following chosen values.
     * @param combinatorIndex The combinator index of the chosen value to delete.
     */
    deleteValue(combinatorIndex) {
        this.props.evaluator.deleteStepThroughOption(combinatorIndex);
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

    /**
     * Handler for the option input's change event.
     */
    handleOptionInputChange(event) {
        event.preventDefault();

        this.setState({
            chosenOption: this.optionInput.value
        });
    }
}