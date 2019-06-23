import React from "react";

import ContractText from "./contract-text.jsx";
import DropDown from "./drop-down.jsx";
import Message from "./message.jsx";

import { acquireSubContract, splitContract } from "./../js/contract-utils.mjs";

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

        var acquisitionTimes = this.populateAcquisitionTimes(props.acquisitionTimes);
        var index = acquisitionTimes.findIndex(elem => elem.unacquired);
        this.state = {
            anytimeIndex: index,
            acquisitionTimes: acquisitionTimes,
            acquisitionError: "",
            acquisitionErrorDetails: ""
        };
    }

    /**
     * Returns the element representing this component.
     */
    render() {
        var anytimeIndexOptions = [];
        for (var i = 0; i < this.state.acquisitionTimes.length; i++) {
            if (this.state.acquisitionTimes[i].unacquired) {
                var index = this.state.acquisitionTimes[i].anytimeIndex;
                anytimeIndexOptions.push(
                    <option key={index} value={index}>
                        {index}
                    </option>
                );
            }
        }

        var combinatorIndex = (this.state.anytimeIndex >= 0 && this.state.acquisitionTimes.length > this.state.anytimeIndex)
            ? this.state.acquisitionTimes[this.state.anytimeIndex].combinatorIndex
            : -1;

        return (
            <div className={AcquireSubContractControls.blockName + "__container"}>
                <div className={AcquireSubContractControls.blockName + "__contract-container"}>
                    <DropDown title="SmartFin Contract">
                            <ContractText
                                contract={this.props.combinatorContract}
                                highlightIndex={combinatorIndex}/>
                        </DropDown>
                </div>

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
            var acquisitionTimes = this.populateAcquisitionTimes(this.props.acquisitionTimes);
            var index = acquisitionTimes.findIndex(elem => elem.unacquired);
            this.setState({
                acquisitionTimes: acquisitionTimes,
                anytimeIndex: index
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
                acquisitionError: "Could not acquire sub-contract. The sub-contract's parent combinator may not yet have been acquired.",
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
     * Gets the available anytime acquisition times.
     */
    populateAcquisitionTimes(acquisitionTimeOptions) {
        if (!this.props.combinatorContract || !acquisitionTimeOptions || acquisitionTimeOptions.length == 0) {
            return [];
        }

        var acquisitionTimes = [];
        var combinators = splitContract(this.props.combinatorContract);
        var index = 0;

        for (var i = 0; i < combinators.length; i++) {
            if (combinators[i].toLowerCase() === "anytime") {
                // Add 1 to account for top-level contract acquisition time
                var time = acquisitionTimeOptions[index + 1]
                var unacquired = !time.isDefined() || new Date(time.getValue() * 1000) > Date.now();
                acquisitionTimes.push(new AnytimeAcquisitionTime(time, index, i, unacquired));
                index++;
            }
        }

        return acquisitionTimes;
    }
}

/**
 * Represents an anytime acquisition time, with the acquisition time option, the
 * anytime-index, the combinator index, and whether or not it has been acquired.
 */
class AnytimeAcquisitionTime {
    /**
     * Initialises a new instance of this class.
     * @param acquisitionTime The acquisition time option.
     * @param anytimeIndex The anytime index.
     * @param combinatorIndex The combinator index.
     * @param unacquired Whether or not this anytime combinator has not been acquired.
     */
    constructor(acquisitionTime, anytimeIndex, combinatorIndex, unacquired) {
        this.acquisitionTime = acquisitionTime;
        this.anytimeIndex = anytimeIndex;
        this.combinatorIndex = combinatorIndex;
        this.unacquired = unacquired;
    }
}