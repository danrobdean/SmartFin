import React from "react";

import ErrorMsg from "./error-msg.jsx";
import Modal from "./modal.jsx";

import { dateToUnixTimestamp } from "./../js/contract-utils.mjs";

/**
 * The contract composition component.
 */
export default class Composition extends React.Component {
    /**
     * The CSS block name.
     */
    static blockName = "composition";

    /**
     * The time input.
     */
    timeInput;

    /**
     * The date input.
     */
    dateInput;

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
            error: false,
            timeErrorMsg: ""
        }
    }

    /**
     * Returns the element representing this component.
     */
    render() {
        return (
            <div className={Composition.blockName + "__main-container"}>
                <Modal title="Help" closeModal={() => this.closeModals()} visible={this.state.helpOpen}>
                    <span className={Composition.blockName + "__help-text"}>
                        First, write a contract in the text box.
                        <br/>
                        <br/>
                        If the contract concerns time, press the <em>Select Time</em> button to input a time into the contract at the cursor's position.
                        <br/>
                        <br/>
                        Once the contract is complete, press the <em>Submit Contract</em> button to deploy it to the blockchain.
                    </span>
                </Modal>

                <Modal title="Select Time" closeModal={() => this.closeModals()} visible={this.state.timePickerOpen}>
                    <div className={Composition.blockName + "__select-time-container"}>
                        <div className={Composition.blockName + "__time-row-container"}>
                            <div className={Composition.blockName + "__time-left-container"}>

                                <span className={Composition.blockName + "__select-time-text"}>
                                    Select a date and time:
                                </span>

                                <div className={Composition.blockName + "__time-input-container"}>
                                    <input
                                        className={Composition.blockName + "__date-input"}
                                        ref={r => this.dateInput = r}
                                        type="date"/>

                                    <input
                                        className={Composition.blockName + "__time-input"}
                                        ref={r => this.timeInput = r}
                                        type="time"/>
                                </div>
                            </div>

                            <div className={Composition.blockName + "__time-right-container"}>
                                <button
                                    className={Composition.blockName + "__insert-time-button"}
                                    onClick={() => this.insertTime()}>
                                    Insert Time
                                </button>
                            </div>
                        </div>

                        <ErrorMsg error={this.state.error}>{this.state.timeErrorMsg}</ErrorMsg>
                    </div>
                </Modal>

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
                            onClick={() => this.submitContract()}>
                            Submit Contract
                        </button>
                    </div>
                </div>
            </div>
        );
    }

    /**
     * Inserts the date/time from the date and time inputs into the contract.
     */
    insertTime() {
        var date = this.dateInput.value;
        var time = this.timeInput.value;
        if (date == "") {
            this.setState({
                timeErrorMsg: "Please select a date.",
                error: true
            });
            return;
        } else if (time == "") {
            this.setState({
                timeErrorMsg: "Please select a time.",
                error: true
            });
            return;
        }

        this.setState({
            timeErrorMsg: "",
            error: false
        });

        this.closeModals(() => {
            var dateTime = new Date(date + " " + time);

            this.closeModals();
            this.compositionInput.focus();
    
            var pos = this.compositionInput.selectionStart;
            this.compositionInput.value =
                this.compositionInput.value.substring(0, pos)
                + dateToUnixTimestamp(dateTime)
                + this.compositionInput.value.substring(pos, this.compositionInput.value.length);
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
     * Closes the modals.
     */
    closeModals(callback = () => {}) {
        this.setState({
            helpOpen: false,
            timePickerOpen: false,
            timeErrorMsg: "",
            error: false
        }, callback);
    }

    /**
     * Submits the combinator contract to the blockchain.
     */
    submitContract() {
        alert("Not implemented.");
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
        // If the key code is ENTER (13), connect to the blockchain
        if ((event.keyCode ? event.keyCode : event.which) == 13) {
            event.preventDefault();
            this.submitContract();
        }
    }
}