import React from "react";

import Message from "./message.jsx";

import { setOrChoice } from "./../js/contract-utils.mjs";

/**
 * The or-choice controls component.
 */
export default class OrChoiceControls extends React.Component {
    /**
     * The CSS block name.
     */
    static blockName = "or-choice-controls";

    /**
     * The or index input.
     */
    orIndexInput;

    /**
     * Initialises a new instance of this class.
     * @param props.address The unlocked account address.
     * @param props.contract The current contract instance.
     * @param props.callback Function to call after setting the or choice.
     */
    constructor(props) {
        super(props);

        this.state = {
            holder: "",
            orIndex: (this.props.orChoices && this.props.orChoices.length > 0) ? 0 : "N/A",
            orChoice: this.getOrChoice(0),
            orError: "",
            orErrorDetails: ""
        };
    }

    /**
     * Returns the element representing this component.
     */
    render() {
        var enabled =
        (
            this.props.contract
            && this.props.holder === this.props.address
            && this.props.orChoices
            && this.props.orChoices.length > 0
            && this.state.orChoice !== undefined
        );

        var orError = this.state.orError;
        if (!this.props.contract) {
            orError = "A contract must be loaded before or-choices can be set.";
        } else if (this.props.holder != this.props.address) {
            orError = "Only the contract holder may set or-choices.";
        } else if (!(this.props.orChoices && this.props.orChoices.length > 0)) {
            orError = <React.Fragment>This contract has no <em>or</em> combinators.</React.Fragment>;
        }

        var selectOptions = [];
        if (this.props.orChoices) {
            for (var i = 0; i < this.props.orChoices.length; i++) {
                if (this.props.orChoices[i].isDefined()) {
                    continue;
                }

                selectOptions.push(
                    <option value={i} key={i}>{i}</option>
                );
            }
        } else {
            selectOptions.push(
                <option value={null} key={0}>N/A</option>
            );
        }

        return (
            <div className={OrChoiceControls.blockName + "__set-or-choice-container"}>
                <div className={OrChoiceControls.blockName + "__set-or-choice-input-container"}>
                    <div className={OrChoiceControls.blockName + "__input-container"}>
                        <span className={OrChoiceControls.blockName + "__label"}>
                            <em>or</em> combinator index:
                        </span>

                        <select
                            className={OrChoiceControls.blockName + "__or-index-input"}
                            ref={r => this.orIndexInput = r}
                            onChange={e => this.handleOrIndexInputChange(e)}>
                            {selectOptions}
                        </select>
                    </div>
                    <div className={OrChoiceControls.blockName + "__input-container"}>
                        <span className={OrChoiceControls.blockName + "__label"}>
                            Selected sub-combinator:
                        </span>

                        <div className={OrChoiceControls.blockName + "__or-value-input"}>
                            <div className={OrChoiceControls.blockName + "__or-value-radio"}>
                                <input
                                    onChange={e => this.handleOrChoiceInputChange(e)}
                                    type="radio"
                                    value={"first"}
                                    name="first"
                                    checked={this.state.orChoice === "first"}/> First
                            </div>

                            <div className={OrChoiceControls.blockName + "__or-value-radio"}>
                                <input
                                    onChange={e => this.handleOrChoiceInputChange(e)}
                                    type="radio"
                                    value={"second"}
                                    name="second"
                                    checked={this.state.orChoice === "second"}/> Second
                            </div>
                        </div>
                    </div>
                </div>

                {Message.renderError(orError, this.state.orErrorDetails, OrChoiceControls.blockName + "__error")}

                <button
                    disabled={!enabled}
                    onClick={() => this.setOrChoice()}>
                    Set Or Choice
                </button>
            </div>
        );
    }

    /**
     * Called when the component has updated.
     * @param prevProps The previous component properties.
     * @param prevState The previous component state.
     */
    componentDidUpdate(prevProps, prevState) {
        // If we receive new or choices, set choice value.
        if (prevProps.orChoices != this.props.orChoices && this.props.orChoices && this.props.orChoices.length > 0) {
            this.setState({
                orIndex: this.props.orChoices.findIndex(elem => !elem.isDefined())
            });
        }

        // If we receive new or index, set choice value
        if (this.state.orIndex != prevState.orIndex && !isNaN(this.state.orIndex)) {
            this.setState({
                orChoice: this.getOrChoice(this.state.orIndex)
            });
        }

        // If contract changes, reset errors
        if (this.props.contract != prevProps.contract) {
            this.setState({
                obsError: "",
                obsErrorDetails: ""
            });
        }
    }

    /**
     * Sets the or-choice on the contract.
     */
    setOrChoice() {
        setOrChoice(this.props.contract, this.props.address, this.state.orIndex, this.state.orChoice === "first").then(() => {
            this.setState({
                orError: "",
                orErrorDetails: ""
            });

            this.props.callback();
        }, err => {
            this.setState({
                orError: "Error occurred while setting the or-choice.",
                orErrorDetails: err.toString()
            })
        });
    }

    /**
     * Handles the or-index input change event.
     */
    handleOrIndexInputChange(event) {
        event.preventDefault();

        this.setState({
            orIndex: this.orIndexInput.value
        });
    }

    /**
     * Handles the or-choice input change event.
     */
    handleOrChoiceInputChange(event) {
        this.setState({
            orChoice: event.target.value
        });
    }

    /**
     * Gets the or choice for the given or-index.
     */
    getOrChoice(index) {
        if (this.props.orChoices && this.props.orChoices.length > index && index > 0) {
            if (this.props.orChoices[index].isDefined()) {
                if (this.props.orChoices[index].getValue() === "true") {
                    return "first";
                } else if (this.props.orChoices[index].getValue() === "false") {
                    return "second";
                }
            }
        }

        return undefined;
    }
}