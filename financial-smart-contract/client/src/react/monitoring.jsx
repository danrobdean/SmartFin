import React from "react";

import DropDown from "./drop-down.jsx";
import Message from "./message.jsx";
import Modal from "./modal.jsx";

import { isSmartContract, getContractAtAddress, getHolder, getCounterParty, getConcluded, getOrChoices, getObsEntries, getAcquisitionTimes } from "./../js/contract-utils.mjs";

/**
 * The contract monitoring component.
 */
export default class Monitoring extends React.Component {
    /**
     * The CSS block name.
     */
    static blockName = "monitoring";

    /**
     * The contract address input.
     */
    contractAddressInput;

    /**
     * Initialises a new instance of this class.
     * @param props.web3 The web3 instance.
     * @param props.address The unlocked account address.
     * @param props.setContract Function to set the current contract instance.
     * @param props.contract The current contract instance.
     */
    constructor(props) {
        super(props);

        this.state = {
            contractRelation: "",
            contractAddress: (this.props.contract) ? this.props.contract.address : "",
            contractAddressError: "",
            contractLoadOpen: !Boolean(this.props.contract),
            contractInteractionError: "",
            contractInteractionErrorDetails: "",
            holder: "N/A",
            counterParty: "N/A",
            concluded: "N/A",
            orChoices: [],
            obsEntries: [],
            acquisitionTimes: []
        };
    }

    /**
     * Returns the element representing this component.
     */
    render() {
        return (
            <div className={Monitoring.blockName + "__main-container"}>
                <Modal title={"Load Contract"} visible={this.state.contractLoadOpen} closeModal={() => this.closeModals()} forceOpen={true}>
                    <div className={Monitoring.blockName + "__load-container"}>
                        <div className={Monitoring.blockName + "__load-input-container"}>
                            <span className={Monitoring.blockName + "__load-contract-label"}>
                                Address of contract to monitor:
                            </span>
                            <input
                                className={Monitoring.blockName + "__load-contract-input"}
                                ref={r => this.contractAddressInput = r}
                                onKeyPress={e => this.handleContractAddressKeyPress(e)}
                                onChange={e => this.handleContractAddressChange(e)}
                                placeholder="0x..."/>
                        </div>

                        {Message.renderError(this.state.contractAddressError, null, Monitoring.blockName + "__load-contract-error")}

                        <button
                            className={Monitoring.blockName + "__load-contract-button"}
                            onClick={() => this.setContractAddress()}>
                            Load Contract
                        </button>
                    </div>
                </Modal>

                <div className={Monitoring.blockName + "__size-container"}>
                    <h2 className={Monitoring.blockName + "__contract-address"}>
                        {this.renderContractHeader()}
                    </h2>

                    {Message.renderError((this.props.contract || this.state.contractLoadOpen) ? null : "Load a contract using the 'Load Contract' button.")}
                    {Message.renderInfo(this.state.contractRelation, null, Monitoring.blockName + "__contract-relation")}
                    {Message.renderError(this.state.contractInteractionError, this.state.contractInteractionErrorDetails, Monitoring.blockName + "__interaction-error")}

                    <div className={Monitoring.blockName + "__contract-interactables"}>
                        <div className={Monitoring.blockName + "__contract-details"}>
                            <div className={Monitoring.blockName + "__basic-details"}>
                                <div className={Monitoring.blockName + "__detail-labels"}>
                                    <span className={Monitoring.blockName + "__detail-label"}>
                                        Holder:
                                    </span>
                                    <span className={Monitoring.blockName + "__detail-label"}>
                                        Counter-party:
                                    </span>
                                    <span className={Monitoring.blockName + "__detail-label"}>
                                        Concluded:
                                    </span>
                                </div>

                                <div className={Monitoring.blockName + "__details"}>
                                    <span className={Monitoring.blockName + "__detail"}>
                                        {this.state.holder}
                                    </span>
                                    <span className={Monitoring.blockName + "__detail"}>
                                        {this.state.counterParty}
                                    </span>
                                    <span className={Monitoring.blockName + "__detail"}>
                                        {this.state.concluded.toString()}
                                    </span>
                                </div>
                            </div>

                            <div className={Monitoring.blockName + "__details-drop-down"}>
                                <DropDown title={"Or choices"}>
                                    {this.renderOrChoices()}
                                </DropDown>
                            </div>

                            <div className={Monitoring.blockName + "__details-drop-down"}>
                                <DropDown title={"Observable values"}>
                                    {this.renderObsValues()}
                                </DropDown>
                            </div>

                            <div className={Monitoring.blockName + "__details-drop-down"}>
                                <DropDown title={"Acquisition Times"}>
                                    {this.renderAcquisitionTimes()}
                                </DropDown>
                            </div>
                        </div>

                        <div className={Monitoring.blockName + "__contract-buttons"}>
                            <button
                                className={Monitoring.blockName + "__contract-button"}
                                onClick={() => this.openLoadContractModal()}>
                                Load Contract
                            </button>
                        </div>
                    </div>
                </div>
            </div>
        );
    }

    /**
     * Initialise the state from the contract.
     */
    componentDidMount() {
        if (this.props.contract) {
            this.initStateFromContract();
        }
    }

    /**
     * Close the contract load modal if a contract is supplied.
     * @param prevProps The previous component props.
     * @param prevState The previous component state.
     */
    componentDidUpdate(prevProps, prevState) {
        if (this.props.contract && this.props.contract != prevProps.contract) {
            this.initStateFromContract();
        }

        // Check the relation of the logged-in user to the contract when the holder/counter-party are loaded.
        if (this.state.holder != prevState.holder && this.state.holder == this.props.address) {
            this.setState({
                contractRelation: "You are the holder of this contract."
            });
        } else if (this.state.counterParty != prevState.counterParty && this.state.counterParty == this.props.address) {
            this.setState({
                contractRelation: "You are the counter-party of this contract."
            });
        }
    }

    /**
     * Returns the element representing the contract header.
     */
    renderContractHeader() {
        if (this.props.contract) {
            return (
                <React.Fragment>
                    Contract deployed at: <em>{this.props.contract.address}</em>
                </React.Fragment>
            );
        } else {
            return (
                <React.Fragment>
                    No contract loaded.
                </React.Fragment>
            );
        }
    }

    /**
     * Returns the element representing the list of or choices.
     */
    renderOrChoices() {
        var orChoiceLabels = [];
        var orChoiceElements = [];
        for (var i = 0; i < this.state.orChoices.length; i++) {
            var orChoice = this.state.orChoices[i];

            orChoiceLabels.push(
                <span key={i} className={Monitoring.blockName + "__detail-label"}>
                    <em>or</em> combinator {i}:
                </span>
            );

            orChoiceElements.push(
                <span key={i} className={Monitoring.blockName + "__detail"}>
                    {(orChoice.isDefined() ? (orChoice.getValue() ? "First child" : "Second child") : "None")}
                </span>
            );
        }

        if (orChoiceLabels.length == 0) {
            orChoiceLabels.push(
                <span key={0} className={Monitoring.blockName + "__detail-label"}>
                    This contract contains no <em>or</em> combinators.
                </span>
            )
        }

        return (
            <div className={Monitoring.blockName + "__basic-details"}>
                <div className={Monitoring.blockName + "__detail-labels"}>
                    {orChoiceLabels}
                </div>
                <div className={Monitoring.blockName + "__details"}>
                    {orChoiceElements}
                </div>
            </div>
        );
    }

    /**
     * Returns the element representing the set of observable values.
     */
    renderObsValues() {
        var obsValueLabels = [];
        var obsValueElements = [];
        var obsArbiterLabels = [];
        var obsArbiterElements = [];

        for (var i = 0; i < this.state.obsEntries.length; i++) {
            var obsEntry = this.state.obsEntries[i];

            obsValueLabels.push(
                <span key={i} className={Monitoring.blockName + "__detail-label"}>
                    Observable value {i}:
                </span>
            );

            obsValueElements.push(
                <span key={i} className={Monitoring.blockName + "__detail"}>
                    {obsEntry.getValue().getValue()}
                </span>
            );

            obsArbiterLabels.push(
                <span key={i} className={Monitoring.blockName + "__detail-label"}>
                    Arbiter:
                </span>
            );

            obsArbiterElements.push(
                <span key={i} className={Monitoring.blockName + "__detail"}>
                    {obsEntry.getAddress()}
                </span>
            );
        }

        if (obsValueLabels.length == 0) {
            obsValueLabels.push(
                <span key={i} className={Monitoring.blockName + "__detail-label"}>
                    This contract contains no observable values.
                </span>
            );
        }

        return (
            <div className={Monitoring.blockName + "__obs-entries"}>
                <div className={Monitoring.blockName + "__basic-details"}>
                    <div className={Monitoring.blockName + "__detail-labels"}>
                        {obsValueLabels}
                    </div>
                    <div className={Monitoring.blockName + "__details"}>
                        {obsValueElements}
                    </div>
                </div>

                <div className={Monitoring.blockName + "__obs-entries-spacer"}/>

                <div className={Monitoring.blockName + "__basic-details"}>
                    <div className={Monitoring.blockName + "__detail-labels"}>
                        {obsArbiterLabels}
                    </div>
                    <div className={Monitoring.blockName + "__details"}>
                        {obsArbiterElements}
                    </div>
                </div>
            </div>
        );
    }

    /**
     * Returns the element representing the list of acquisition times.
     */
    renderAcquisitionTimes() {
        if (this.state.acquisitionTimes.length == 0) {
            return (
                <span className={Monitoring.blockName + "__detail-label"}>
                    N/A
                </span>
            );
        }

        var acquisitionTimeLabels = [
            <span key={0} className={Monitoring.blockName + "__detail-label"}>
                Contract acquisition time:
            </span>
        ];
        var acquisitionTimeElements = [
            <span key={0} className={Monitoring.blockName + "__detail"}>
                {this.state.acquisitionTimes[0].getValue()}
            </span>
        ];

        for (var i = 1; i < this.state.acquisitionTimes.length; i++) {
            var acquisitionTime = this.state.acquisitionTimes[i];

            acquisitionTimeLabels.push(
                <span key={i} className={Monitoring.blockName + "__detail-label"}>
                    <em>anytime</em> combinator {i - 1}:
                </span>
            );

            acquisitionTimeElements.push(
                <span key={i} className={Monitoring.blockName + "__detail"}>
                    {acquisitionTime.getValue()}
                </span>
            );
        }

        return (
            <div className={Monitoring.blockName + "__basic-details"}>
                <div className={Monitoring.blockName + "__detail-labels"}>
                    {acquisitionTimeLabels}
                </div>
                <div className={Monitoring.blockName + "__details"}>
                    {acquisitionTimeElements}
                </div>
            </div>
        );
    }

    /**
     * Initialises the state based on the contract.
     */
    async initStateFromContract() {
        this.setState({
            contractLoadOpen: false
        });

        try {
            var holder = await getHolder(this.props.contract, this.props.address);
            var counterParty = await getCounterParty(this.props.contract, this.props.address);
            var concluded = await getConcluded(this.props.contract, this.props.address);
            var orChoices = await getOrChoices(this.props.contract, this.props.address);
            var obsEntries = await getObsEntries(this.props.contract, this.props.address);
            var acquisitionTimes = await getAcquisitionTimes(this.props.contract, this.props.address);

            this.setState({
                holder: holder,
                counterParty: counterParty,
                concluded: concluded,
                orChoices: orChoices,
                obsEntries: obsEntries,
                acquisitionTimes: acquisitionTimes
            });
        } catch(err) {
            this.setState({
                contractInteractionError: "Contract functions not found. Please check that the contract address is correct.",
                contractInteractionErrorDetails: err
            });
        }
    }

    /**
     * Sets the contract instance address.
     */
    setContractAddress() {
        isSmartContract(this.state.contractAddress).then(() => {
            this.props.setContract(getContractAtAddress(this.state.contractAddress));
        }, err => {
            this.setState({
                contractAddressError: err
            });
        });
    }

    /**
     * Opens the load contract modal.
     */
    openLoadContractModal() {
        this.setState({
            contractLoadOpen: true
        });
    }

    /**
     * Handles the contract address input key press event.
     */
    handleContractAddressKeyPress(event) {
        // If the key code is ENTER (13), display deploy modal
        if ((event.keyCode ? event.keyCode : event.which) == 13) {
            event.preventDefault();
            this.setContractAddress();
        }
    }

     /**
      * Handles the contract address input change event.
      */
     handleContractAddressChange(event) {
        event.preventDefault();

        this.setState({
            contractAddress: this.contractAddressInput.value
        });
    }

    /**
     * Closes all modals.
     */
    closeModals() {
        this.setState({
            contractLoadOpen: false
        });
    }
}