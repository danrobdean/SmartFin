import React from "react";

import DropDown from "./drop-down.jsx";
import Message from "./message.jsx";

import { setOrChoice, splitContract } from "./../js/contract-utils.mjs";
import ContractText from "./contract-text.jsx";

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
     * @param props.combinatorContract The combinator contract.
     * @param props.callback Function to call after setting the or choice.
     * @param props.orChoices The set of or choice options.
     */
    constructor(props) {
        super(props);

        var orChoices = this.populateOrChoices(props.orChoices);
        var index = orChoices.findIndex(elem => !elem.choice.isDefined());
        this.state = {
            holder: "",
            orIndex: index,
            orChoice: true,
            orChoices: orChoices,
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
            && this.state.orChoices
            && this.state.orChoices.length > 0
        );

        var orError = this.state.orError;
        if (!this.props.contract) {
            orError = "A contract must be loaded before or-choices can be set.";
        } else if (this.props.holder != this.props.address) {
            orError = "Only the contract holder may set or-choices.";
        } else if (!(this.state.orChoices && this.state.orChoices.length > 0)) {
            orError = <React.Fragment>This contract has no <em>or</em> combinators.</React.Fragment>;
        }

        var selectOptions = [];
        if (this.state.orChoices) {
            for (var i = 0; i < this.state.orChoices.length; i++) {
                if (this.state.orChoices[i].choice.isDefined()) {
                    continue;
                }

                var index = this.state.orChoices[i].orIndex;
                selectOptions.push(
                    <option value={index} key={index}>{index}</option>
                );
            }
        } else {
            selectOptions.push(
                <option value={null} key={0}>N/A</option>
            );
        }

        var combinatorIndex = (this.state.orIndex >= 0 && this.state.orChoices.length > this.state.orIndex)
            ? this.state.orChoices[this.state.orIndex].combinatorIndex
            : -1;

        return (
            <div className={OrChoiceControls.blockName + "__set-or-choice-container"}>
                <div className={OrChoiceControls.blockName + "__contract-container"}>
                    <DropDown title="SmartFin Contract">
                            <ContractText
                                contract={this.props.combinatorContract}
                                highlightIndex={combinatorIndex}/>
                        </DropDown>
                </div>

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
                            <label className={OrChoiceControls.blockName + "__or-value-radio"}>
                                <input
                                    onChange={e => this.handleOrChoiceInputChange(e)}
                                    type="radio"
                                    value="first"
                                    name="first"
                                    checked={this.state.orChoice}/> First
                            </label>

                            <label className={OrChoiceControls.blockName + "__or-value-radio"}>
                                <input
                                    onChange={e => this.handleOrChoiceInputChange(e)}
                                    type="radio"
                                    value="second"
                                    name="second"
                                    checked={!this.state.orChoice}/> Second
                            </label>
                        </div>
                    </div>
                </div>

                {Message.renderError(orError, this.state.orErrorDetails, OrChoiceControls.blockName + "__error")}

                <button
                    disabled={!enabled}
                    onClick={() => this.setOrChoiceOnContract()}>
                    Set Or Choice
                </button>
            </div>
        );
    }

    /**
     * Called when the component has updated.
     * @param prevProps The previous component properties.
     */
    componentDidUpdate(prevProps) {
        // If we receive new or choices, set choice value.
        if (this.props.combinatorContract != prevProps.combinatorContract || prevProps.orChoices != this.props.orChoices) {
            var orChoices = this.populateOrChoices(this.props.orChoices);
            var index = orChoices.findIndex(elem => !elem.choice.isDefined());

            this.setState({
                orIndex: index,
                orChoices: orChoices
            });
        }

        // If contract changed, reset errors
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
    setOrChoiceOnContract() {
        setOrChoice(this.props.contract, this.props.address, this.state.orIndex, this.state.orChoice).then(() => {
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
            orChoice: event.target.value === "first"
        });
    }

    // Creates a list of OrChoice objects from the given set of or-choice options.
    populateOrChoices(orChoiceOptions) {
        if (!this.props.combinatorContract || !orChoiceOptions) {
            return [];
        }

        var orChoices = [];
        var combinators = splitContract(this.props.combinatorContract);
        var orIndex = 0;

        for (var i = 0; i < combinators.length; i++) {
            if (combinators[i] === "or") {
                orChoices.push(new OrChoice(orChoiceOptions[orIndex], orIndex, i));
                orIndex++;
            }
        }

        return orChoices;
    }
}

/**
 * Represents an or-choice, with a choice value, an or-index, and a combinator index.
 */
class OrChoice {
    /**
     * Initialises a new instance of this class.
     * @param choice The or choice.
     * @param orIndex The or-index.
     * @param combinatorIndex The combinator index.
     */
    constructor(choice, orIndex, combinatorIndex) {
        this.choice = choice;
        this.orIndex = orIndex;
        this.combinatorIndex = combinatorIndex;
    }
}