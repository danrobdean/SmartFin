import React from "react";

import DropDown from "./drop-down.jsx";
import LoadContractControls from "./load-contract-controls.jsx";
import Message from "./message.jsx";
import Modal from "./modal.jsx";
import ObsValueControls from "./obs-value-controls.jsx";
import OrChoiceControls from "./or-choice-controls.jsx";

import { getHolder, getCounterParty, getConcluded, getOrChoices, getObsEntries, getAcquisitionTimes } from "./../js/contract-utils.mjs";

/**
 * The contract monitoring component.
 */
export default class Monitoring extends React.Component {
    /**
     * The CSS block name.
     */
    static blockName = "monitoring";

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
            contractLoadOpen: !Boolean(this.props.contract),
            orChoiceOpen: false,
            obsValueOpen: false,
            contractInteractionError: "",
            contractInteractionErrorDetails: "",
            holder: "N/A",
            counterParty: "N/A",
            concluded: "N/A",
            orChoices: [],
            obsEntries: [],
            acquisitionTimes: [],
        };
    }

    /**
     * Returns the element representing this component.
     */
    render() {
        var orChoicesDisabled = this.state.holder !== this.props.address
            || this.state.concluded
            || !(this.state.orChoices && this.state.orChoices.length > 0);
        var setObsValueDisabled = true;
        if (!this.state.concluded) {
            for (var entry of this.state.obsEntries) {
                if (this.props.address === entry.getAddress()) {
                    setObsValueDisabled = false;
                    break;
                }
            }
        }

        return (
            <div className={Monitoring.blockName + "__main-container"}>
                <Modal title="Load Contract" visible={this.state.contractLoadOpen} closeModal={() => this.closeModals()} forceOpen={true}>
                    <LoadContractControls setContract={c => this.props.setContract(c)}/>
                </Modal>

                <Modal title={"Set Or Choice"} visible={this.state.orChoiceOpen} closeModal={() => this.closeModals()}>
                    <OrChoiceControls
                        contract={this.props.contract}
                        holder={this.state.holder}
                        address={this.props.address}
                        orChoices={this.state.orChoices}
                        callback={() => this.closeModals()}/>
                </Modal>

                <Modal title="Set Observable Value" visible={this.state.obsValueOpen} closeModal={() => this.closeModals()}>
                    <ObsValueControls
                        contract={this.props.contract}
                        address={this.props.address}
                        obsEntries={this.state.obsEntries}
                        callback={() => this.closeModals()}/>
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

                            <button
                                className={Monitoring.blockName + "__contract-button"}
                                onClick={() => this.openOrChoiceModal()}
                                disabled={orChoicesDisabled}>
                                Set Or Choices
                            </button>

                            <button
                                className={Monitoring.blockName + "__contract-button"}
                                onClick={() => this.openObsValueModal()}
                                disabled={setObsValueDisabled}>
                                    Set Observable Value
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
                contractInteractionErrorDetails: err.toString()
            });
        }
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
     * Opens the or-choice modal.
     */
    openOrChoiceModal() {
        this.setState({
            orChoiceOpen: true
        });
    }

    /**
     * Opens the obs-value modal.
     */
    openObsValueModal() {
        this.setState({
            obsValueOpen: true
        });
    }

    /**
     * Closes all modals.
     */
    closeModals() {
        this.setState({
            contractLoadOpen: false,
            orChoiceOpen: false,
            obsValueOpen: false
        });
    }
}