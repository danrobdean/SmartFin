import React from "react";

import Message from "./message.jsx";

import { isSmartContract, getContractAtAddress } from "./../js/contract-utils.mjs";

/**
 * The load contract controls component.
 */
export default class LoadContractControls extends React.Component {
    /**
     * The CSS block name.
     */
    static blockName = "load-contract-controls";

    /**
     * The contract address input.
     */
    contractAddressInput;

    /**
     * Initialises a new instance of this class.
     * @param props.setContract Function to set the current contract instance.
     */
    constructor(props) {
        super(props);

        this.state = {
            contractAddressError: ""
        };
    }

    /**
     * Returns the element representing this component.
     */
    render() {
        return (
            <div className={LoadContractControls.blockName + "__load-container"}>
                <div className={LoadContractControls.blockName + "__load-input-container"}>
                    <span className={LoadContractControls.blockName + "__load-contract-label"}>
                        Address of contract to monitor:
                    </span>
                    <input
                        className={LoadContractControls.blockName + "__load-contract-input"}
                        ref={r => this.contractAddressInput = r}
                        onKeyPress={e => this.handleContractAddressKeyPress(e)}
                        onChange={e => this.handleContractAddressChange(e)}
                        placeholder="0x..."/>
                </div>

                {Message.renderError(this.state.contractAddressError, null, LoadContractControls.blockName + "__load-contract-error")}

                <button
                    className={LoadContractControls.blockName + "__load-contract-button"}
                    onClick={() => this.setContractAddress()}>
                    Load Contract
                </button>
            </div>
        );
    }

    /**
     * Focus on the contract address input upon mounting.
     */
    componentDidMount() {
        this.contractAddressInput.focus();
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
     * Handles the contract address input change event.
     */
    handleContractAddressChange(event) {
        event.preventDefault();

        this.setState({
            contractAddress: this.contractAddressInput.value
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
}