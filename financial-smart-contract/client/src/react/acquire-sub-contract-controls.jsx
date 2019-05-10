import React from "react";

import Message from "./message.jsx";

import { acquireSubContract } from "./../js/contract-utils.mjs";

/**
 * Component representing the acquire controls for a contract.
 */
export default class AcquireSubContractControls extends React.Component {
    /**
     * The CSS block name for this component.
     */
    static blockName = "acquire-sub-contract-controls";

    /**
     * The anytime combinator index input.
     */
    anytimeIndexInput;

    /**
     * Initialises a new instance of this class.
     * @param props.address The unlocked account address.
     * @param props.contract The current contract instance.
     * @param props.acquisitionTimes The contract's acquisition times.
     * @param props.callback Function to call after acquiring the sub-contract.
     */
    constructor(props) {
        super(props);

        var acquirableIndexes = this.getAcquirableIndexes(props.acquisitionTimes);
        this.state = {
            anytimeIndex: (acquirableIndexes.length > 0) ? acquirableIndexes[0] : "N/A",
            acquisitionError: "",
            acquisitionErrorDetails: ""
        };
    }

    /**
     * Returns the element representing this component.
     */
    render() {
        var anytimeIndexOptions = [];
        var acquirableIndexes = this.getAcquirableIndexes(this.props.acquisitionTimes);
        for (var index of acquirableIndexes) {
            anytimeIndexOptions.push(
                <option key={index} value={index}>
                    {index}
                </option>
            );
        }

        return (
            <div className={AcquireSubContractControls.blockName + "__container"}>
                <div className={AcquireSubContractControls.blockName + "__input-container"}>
                    <span className={AcquireSubContractControls.blockName + "__label"}>
                        <em>anytime</em> combinator index:
                    </span>
                    
                    <select
                        className={AcquireSubContractControls.blockName + "__input"}
                        ref={r => this.anytimeIndexInput = r}
                        onChange={e => this.handleAnytimeIndexInputChange(e)}
                        value={this.state.anytimeIndex}>
                        {anytimeIndexOptions}
                    </select>
                </div>

                {Message.renderError(this.state.acquisitionError, this.state.acquisitionErrorDetails, AcquireSubContractControls.blockName + "__error")}

                <button
                    className={AcquireSubContractControls.blockName + "__acquire-button"}
                    onClick={() => this.acquireSubContract()}>
                    Acquire Sub-contract
                </button>
            </div>
        );
    }

    /**
     * Called after the component receives new properties/state.
     * @param prevProps The previous component properties.
     */
    componentDidUpdate(prevProps) {
        if (this.props.acquisitionTimes != prevProps.acquisitionTimes) {
            var acquirableIndexes = this.getAcquirableIndexes(this.props.acquisitionTimes);
            this.setState({
                anytimeIndex: (acquirableIndexes.length > 0) ? acquirableIndexes[0] : "N/A"
            });
        }
    }

    /**
     * Called when the acquire button is pressed, attempts to acquire the sub-contract.
     */
    acquireSubContract() {
        acquireSubContract(this.props.contract, this.props.address, this.state.anytimeIndex).then(() => {
            this.setState({
                acquisitionError: "",
                acquisitionErrorDetails: null
            });

            this.props.callback();
        }, err => {
            this.setState({
                acquisitionError: "Could not acquire sub-contract. Please ensure that sub-contract's parent has been acquired, and the sub-contract's acquisition date is in the future (if one exists).",
                acquisitionErrorDetails: err.toString()
            })
        });
    }

    /**
     * Handles the change event on the anytime index input.
     */
    handleAnytimeIndexInputChange(event) {
        event.preventDefault();

        this.setState({
            anytimeIndex: this.anytimeIndexInput.value
        });
    }

    /**
     * Gets the available anytime combinator indexes.
     */
    getAcquirableIndexes(acquisitionTimes) {
        var acquirableIndexes = [];

        if (acquisitionTimes) {
            for (var i = 1; i < acquisitionTimes.length; i++) {
                if (!acquisitionTimes[i].isDefined() || new Date(acquisitionTimes[i].getValue() * 1000) > Date.now()) {
                    acquirableIndexes.push(i - 1);
                }
            }
        }

        return acquirableIndexes;
    }
}