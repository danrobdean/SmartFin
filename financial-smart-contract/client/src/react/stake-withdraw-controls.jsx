import React from "react";

import Message from "./message.jsx";

import { stake, withdraw } from "./../js/contract-utils.mjs";

export default class StakeWithdrawControls extends React.Component {
    /**
     * The CSS block name for this component.
     */
    static blockName = "stake-withdraw-controls";

    /**
     * The amount input.
     */
    amountInput;

    /**
     * Initialises a new instance of this class.
     * @param props.stake Whether or not this component represents the stake or withdraw controls.
     * @param props.contract The smart contract instance.
     * @param props.address The address of the logged-in account.
     * @param props.callback The callback function to call once the stake/withdrawal is complete.
     */
    constructor(props) {
        super(props);

        this.state = {
            amount: 0,
            contractError: "",
            contractErrorDetails: null
        };
    }

    /**
     * Returns the element representing this component.
     */
    render() {
        return (
            <div className={StakeWithdrawControls.blockName + "__container"}>
                <div className={StakeWithdrawControls.blockName + "__input-container"}>
                    <span className={StakeWithdrawControls.blockName + "__label"}>
                        Amount:
                    </span>

                    <input
                        className={StakeWithdrawControls.blockName + "__input"}
                        ref={r => this.amountInput = r}
                        onChange={e => this.handleAmountInputChange(e)}
                        onKeyPress={e => this.handleAmountInputKeyPress(e)}
                        placeholder="Wei"/>
                </div>

                {Message.renderError(this.state.contractError, this.state.contractErrorDetails, StakeWithdrawControls.blockName + "__error")}

                <button
                    className={StakeWithdrawControls.blockName + "__button"}
                    onClick={() => this.submit()}>
                    {(this.props.stake) ? "Stake Wei" : "Withdraw Wei"}
                </button>
            </div>
        );
    }

    /**
     * Submits the stake/withdraw request to the contract.
     */
    submit() {
        const successCallback = () => {
            this.setState({
                contractError: "",
                contractErrorDetails: ""
            })

            this.props.callback();
        };

        const failCallback = (err, msg) => {
            this.setState({
                contractError: msg,
                contractErrorDetails: err.toString()
            });
        };

        if (this.props.stake) {
            stake(this.props.contract, this.props.address, this.state.amount).then(successCallback,
                err => failCallback(err, "Could not stake funds in contract. Please ensure the account has the required balance.")
            );
        } else {
            withdraw(this.props.contract, this.props.address, this.state.amount).then(successCallback,
                err => failCallback(err, "Could not withdraw funds in contract. Please ensure the party has the required balance, the contract has enough total funds, and that the gas cost (2300 Wei) is not higher than the withdrawal amount.")
            );
        }
    }

    /**
     * Handles the change event in the amount input.
     */
    handleAmountInputChange(event) {
        event.preventDefault();

        this.setState({
            amount: this.amountInput.value
        });
    }

    /**
     * Handles the key press event in the amount input.
     */
    handleAmountInputKeyPress(event) {
        // If the key code is ENTER (13), submit value
        if ((event.keyCode ? event.keyCode : event.which) == 13) {
            event.preventDefault();
            this.submit();
        }
    }
}