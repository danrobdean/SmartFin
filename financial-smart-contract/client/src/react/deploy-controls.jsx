import React from "react";

import Message from "./message.jsx";

import { isValidAddress, serializeCombinatorContract, loadAndDeployContract } from "./../js/contract-utils.mjs";

/**
 * Component representing controls for deploying a contract.
 */
export default class DeployControls extends React.Component {
    /**
     * The CSS block name for this component.
     */
    static blockName = "deploy-controls";

    /**
     * The holder input.
     */
    holderInput;

    /**
     * Initialises a new instance of this class.
     * @param props.contract The combinator contract.
     * @param props.warning The contract warning.
     */
    constructor(props) {
        super(props);

        this.state = {
            holder: "",
            holderError: ""
        };
    }

    /**
     * Returns the element representing this component.
     */
    render() {
        return (
            <div className={DeployControls.blockName + "__container"}>
                <div className={DeployControls.blockName + "__details-container"}>
                    <span className={DeployControls.blockName + "__contract-label"}>  
                        Contract to deploy:
                    </span>

                    <span className={DeployControls.blockName + "__contract"}>
                        <em>{this.props.contract}</em>
                    </span>

                    {Message.renderWarning(this.props.warning)}

                    <span className={DeployControls.blockName + "__holder-label"}>
                        The contract holder:
                    </span>

                    <input
                        className={DeployControls.blockName + "__holder-input"}
                        ref={r => this.holderInput = r}
                        onKeyPress={e => this.handleHolderKeyPress(e)}
                        onChange={e => this.handleHolderChange(e)}
                        placeholder="0x..."
                        rows={50}/>

                    {Message.renderInfo("The contract holder is the only account that has the ability to acquire the contract.")}
                    {Message.renderError(this.state.holderError)}
                </div>
                <div className={DeployControls.blockName + "__button-container"}>
                    <button
                        className={DeployControls.blockName + "__deploy-button"}
                        onClick={() => this.deployContract()}>
                        Deploy
                    </button>
                </div>
            </div>
        );
    }

    /**
     * Deploys the combinator contract to the blockchain.
     */
    deployContract() {
        if (!isValidAddress(this.state.holder)) {
            this.setState({
                holderError: "'" + this.state.holder + "' is not a valid address."
            });
            return;
        } else {
            this.setState({
                holderError: ""
            });
        }

        var serializedContract = serializeCombinatorContract(this.props.contract);
        loadAndDeployContract(serializedContract, this.state.holder, this.props.address);
    }

    /**
     * Deploys the contract on enter being pressed.
     * @param event The key press event.
     */
    handleHolderKeyPress(event) {
        // If the key code is ENTER (13), select the time
        if ((event.keyCode ? event.keyCode : event.which) == 13) {
            event.preventDefault();
            this.deployContract();
        }
    }

    handleHolderChange(event) {
        event.preventDefault();

        this.setState({
            holder: this.holderInput.value
        });
    }
}