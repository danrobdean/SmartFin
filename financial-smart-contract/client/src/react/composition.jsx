import React from "react";

import DeployControls from "./deploy-controls.jsx";
import Help from "./help.jsx";
import Message from "./message.jsx";
import Modal from "./modal.jsx";
import TimeSelect from "./time-select.jsx";

import { verifyContract } from "./../js/contract-utils.mjs"

/**
 * The contract composition component.
 */
export default class Composition extends React.Component {
    /**
     * The CSS block name.
     */
    static blockName = "composition";

    /**
     * The contract composition input. 
     */
    compositionInput;

    /**
     * Initialises a new instance of this class.
     * @param props.web3 The web3 instance.
     * @param props.address The unlocked account address.
     */
    constructor(props) {
        super(props);

        this.state = {
            contract: "",
            helpOpen: false,
            timePickerOpen: false,
            deployOpen: false,
            contractWarning: "",
            contractError: "",
            contractErrorStack: ""
        }
    }

    /**
     * Returns the element representing this component.
     */
    render() {
        return (
            <div className={Composition.blockName + "__main-container"}>
                <Modal title="Help" closeModal={() => this.closeModals()} visible={this.state.helpOpen}>
                    <Help/>
                </Modal>

                <Modal title="Select Time" closeModal={() => this.closeModals()} visible={this.state.timePickerOpen}>
                    <TimeSelect returnTime={unixTime => this.insertTime(unixTime)}/>
                </Modal>

                <Modal title="Deploy Contract" closeModal={() => this.closeModals()} visible={this.state.deployOpen}>
                    <DeployControls address={this.props.address} contract={this.state.contract} warning={this.state.contractWarning}/>
                </Modal>
                <div className={Composition.blockName + "__size-container"}>
                    <div className={Composition.blockName + "__wrapping-container"}>
                        <div className={Composition.blockName + "__composition-container"}>
                            <span className={Composition.blockName + "__composition-title"}>Enter your contract here:</span>
                            <textarea
                                className={Composition.blockName + "__composition-input"}
                                ref={r => this.compositionInput = r}
                                onChange={e => this.handleCompositionInputChange(e)}
                                onKeyPress={e => this.handleCompositionInputKeyPress(e)}
                                rows={15}
                                cols={70}/>
                        </div>
                        <div className={Composition.blockName + "__controls-container"}>
                            <button
                                className={Composition.blockName + "__control-button"}
                                onClick={() => this.displayHelp()}>
                                Help
                            </button>
                            <button
                                className={Composition.blockName + "__control-button"}
                                onClick={() => this.displayTimePicker()}>
                                Select Time
                            </button>
                            <button
                                className={Composition.blockName + "__control-button"}
                                onClick={() => this.displayDeploy()}>
                                Deploy Contract
                            </button>
                        </div>
                    </div>
                    {Message.renderError(this.state.contractError, this.state.contractErrorStack)}
                </div>
            </div>
        );
    }

    /**
     * Inserts the date/time from the date and time inputs into the contract.
     */
    insertTime(unixTime) {
        this.closeModals(() => {
            this.compositionInput.focus();
    
            var pos = this.compositionInput.selectionStart;
            this.compositionInput.value =
                this.compositionInput.value.substring(0, pos)
                + unixTime
                + this.compositionInput.value.substring(pos, this.compositionInput.value.length);
            this.compositionInput.setSelectionRange(pos + unixTime.toString().length, pos + unixTime.toString().length);

            this.setState({
                contract: this.compositionInput.value
            });
        });
    }

    /**
     * Displays the help modal.
     */
    displayHelp() {
        this.setState({
            helpOpen: true,
            timePickerOpen: false
        });
    }

    /**
     * Displays the time picker modal.
     */
    displayTimePicker() {
        this.setState({
            timePickerOpen: true,
            helpOpen: false
        });
    }

    /**
     * Displays the deploy contract modal.
     */
    displayDeploy() {
        var res = verifyContract(this.state.contract);

        if (res.error) {
            this.setState({
                contractWarning: "",
                contractError: res.error,
                contractErrorStack: res.stack
            });
        } else {
            this.setState({
                contractWarning: res.warning,
                contractError: "",
                contractErrorStack: "",
                deployOpen: true
            });
        }
    }

    /**
     * Closes the modals.
     * @param callback Called once the modals have been closed.
     */
    closeModals(callback = () => {}) {
        this.setState({
            helpOpen: false,
            timePickerOpen: false,
            deployOpen: false
        }, callback);
    }

    /**
     * Handles the change event for the contract composition input.
     * @param event The change event.
     */
    handleCompositionInputChange(event) {
        event.preventDefault();

        this.setState({
            contract: this.compositionInput.value
        });
    }

    /**
     * Handles the key press event for the contract composition input.
     * @param event The key press event.
     */
    handleCompositionInputKeyPress(event) {
        // If the key code is ENTER (13), display deploy modal
        if ((event.keyCode ? event.keyCode : event.which) == 13) {
            event.preventDefault();
            this.displayDeploy();
        }
    }
}