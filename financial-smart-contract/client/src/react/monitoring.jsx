import React from "react";

import Message from "./message.jsx";
import Modal from "./modal.jsx";

import { web3, isSmartContract, getContractAtAddress } from "./../js/contract-utils.mjs";

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
            counterParty: "N/A"
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
                            <div className={Monitoring.blockName + "__detail-labels"}>
                                <span className={Monitoring.blockName + "__detail-label"}>
                                Holder:
                                </span>
                                <span className={Monitoring.blockName + "__detail-label"}>
                                Counter-party:
                                </span>
                            </div>

                            <div className={Monitoring.blockName + "__details"}>
                                <span className={Monitoring.blockName + "__detail"}>
                                {this.state.holder}
                                </span>
                                <span className={Monitoring.blockName + "__detail"}>
                                {this.state.counterParty}
                                </span>
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
     */
    componentDidUpdate(prevProps) {
        if (this.props.contract && this.props.contract != prevProps.contract) {
            this.initStateFromContract();
        }
    }

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
     * Initialises the state based on the contract.
     */
    async initStateFromContract() {
        this.setState({
            contractLoadOpen: false
        });

        const handleError = (details) => {
            this.setState({
                contractInteractionError: "Contract functions not found. Please check that the contract address is correct.",
                contractInteractionErrorDetails: details
            });
        };
        const contractMethods = this.props.contract.methods;
        const callData = { from: this.props.address };
        var contractRelation = this.state.contractRelation;

        var holder = (await contractMethods.get_holder().call(callData).catch(handleError)).returnValue0;
        var counterParty = (await contractMethods.get_counter_party().call(callData).catch(handleError)).returnValue0;

        if (holder == this.props.address) {
            contractRelation = "You are the holder of this contract.";
        } else if (counterParty == this.props.address) {
            contractRelation = "You are the counter-party of this contract.";
        }

        this.setState({
            contractRelation: contractRelation,
            holder: holder,
            counterParty: counterParty
        });
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