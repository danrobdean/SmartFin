import React from "react";

import Message from "./message.jsx";
import Spinner from "./spinner.jsx";

import { setupWeb3, unlockAccount } from "./../js/contract-utils.mjs";

const LOCAL_BLOCKCHAIN_URL = "http://localhost:8545";

// The UI for connecting an account via MetaMask or direct input.
export default class ConnectAccount extends React.Component {
    // The timeout for connecting to a blockchain manually.
    static CONNECT_TIMEOUT = 10000;

    // The timeout for unlocking an account manually.
    static UNLOCK_TIMEOUT = 10000;

    // The CSS block name for this component
    static blockName = "connect-account";

    // Reference to the manual account address input.
    addressInput;

    // Reference to the manual account password input.
    passwordInput;

    // Reference to the manual blockchain URL input.
    urlInput;

    // Set of scheduled timeouts.
    timeouts = [];

    /**
     * Initialises an instance of this class.
     * @param props.setWeb3 Function to set the web3 instance.
     */
    constructor(props) {
        super(props);

        this.state = {
            address: "",
            password: "",
            url: LOCAL_BLOCKCHAIN_URL,
            web3: null,
            manualConnectError: "",
            manualAccountUnlockError: "",
            connecting: false,
            unlocking: false
        }
    }

    render() {
        var manualAccountClassModifier = (this.state.web3 == null) ? "--hidden" : "";
        return (
            <div className={ConnectAccount.blockName + "__main-container"}>
                <div className={ConnectAccount.blockName + "__metamask-container"}>
                    <span className={ConnectAccount.blockName + "__metamask-title"}>Click here to connect to MetaMask:</span>

                    <button
                        className={ConnectAccount.blockName + "__metamask-connect-button"}
                        onClick={() => this.connectMetaMask()}>
                        Connect to MetaMask
                    </button>
                </div>
                <div className={ConnectAccount.blockName + "__manual-container"}>
                    <span className={ConnectAccount.blockName + "__manual-title"}>...or manually enter the details of your blockchain and account here (NOT RECOMMENDED):</span>
                    <div className={ConnectAccount.blockName + "__manual_input_align_container"}>
                        <div className={ConnectAccount.blockName + "__manual-url-container"}>
                            <div className={ConnectAccount.blockName + "__manual-labels-inputs-container"}>
                                <div className={ConnectAccount.blockName + "__manual-label-container"}>
                                    <span className={ConnectAccount.blockName + "__manual-url-label"}>Blockchain URL: </span>
                                </div>
                                <div className={ConnectAccount.blockName + "__manual-input-container"}>
                                    <input
                                        className={ConnectAccount.blockName + "__manual-url-input"}
                                        ref={r => this.urlInput = r}
                                        onChange={e => this.handleUrlInputChange(e)}
                                        onKeyPress={e => this.handleUrlInputKeyPress(e)}
                                        placeholder={LOCAL_BLOCKCHAIN_URL}
                                        size={45}
                                        name="url"
                                        autoComplete="off"/>
                                </div>
                            </div>

                            <button
                                className={ConnectAccount.blockName + "__manual-connect-button"}
                                onClick={() => this.connectManual()}>
                                Connect Manually
                            </button>

                            {Spinner.renderNotice((this.state.connecting) ? "Connecting" : null)}
                            {Message.renderError(this.state.manualConnectError)}
                        </div>
                        <div className={ConnectAccount.blockName + "__manual-account-details-container" + manualAccountClassModifier}>
                            <div className={ConnectAccount.blockName + "__manual-labels-inputs-container"}>
                                <div className={ConnectAccount.blockName + "__manual-label-container"}>
                                    <span className={ConnectAccount.blockName + "__manual-address-label"}>Address: </span>
                                    <span className={ConnectAccount.blockName + "__manual-password-label"}>Password: </span>
                                </div>

                                <div className={ConnectAccount.blockName + "__manual-input-container"}>
                                    <input
                                        className={ConnectAccount.blockName + "__manual-address-input"}
                                        ref={r => this.addressInput = r}
                                        onChange={e => this.handleAddressInputChange(e)}
                                        onKeyPress={e => this.handleAccountInputKeyPress(e)}
                                        placeholder="0x..."
                                        size={45}
                                        name="address"/>
                                    <input
                                        className={ConnectAccount.blockName + "__manual-password-input"}
                                        ref={r => this.passwordInput = r}
                                        onChange={e => this.handlePasswordInputChange(e)}
                                        onKeyPress={e => this.handleAccountInputKeyPress(e)}
                                        type="password"
                                        name="password"
                                        size={45}/>
                                </div>
                            </div>

                            <button
                                className={ConnectAccount.blockName + "__manual-unlock-account-button"}
                                onClick={() => this.unlockAccountManual()}>
                                Unlock Account
                            </button>

                            {Spinner.renderNotice((this.state.unlocking) ? "Unlocking" : null)}
                            {Message.renderError(this.state.manualAccountUnlockError)}
                        </div>
                    </div>
                </div>
            </div>
        );
    }

    /**
     * Cancel all scheduled callbacks.
     */
    componentWillUnmount() {
        for (var timeout of this.timeouts) {
            clearTimeout(timeout);
        }
    }

    /**
     * Attempts to connect to the MetaMask instance.
     */
    connectMetaMask() {
        var web3 = setupWeb3(true);
        this.checkConnection(web3, true);
    }

    /**
     * Attempts to connect to the manually input blockchain.
     */
    connectManual() {
        // Setup web3 connection
        var web3 = setupWeb3(false, this.state.url);
        this.checkConnection(web3, false);
    }

    checkConnection(web3, metamask) {
        var successfulState = {
            web3: web3,
            manualConnectError: "",
            connecting: false
        };
        var failedState = (errorMsg) => {return {
            web3: null,
            manualConnectError: errorMsg,
            connecting: false
        }}

        this.setState({
            manualConnectError: "",
            connecting: true
        }, () => {
            // Set connection timeout.
            var timeout = setTimeout(() => {
                if (this.state.connecting) {
                    // Connection timed out, set failed.
                    this.setState(failedState("Manual connection timed out! Please check the blockchain URL."));
                }
            }, ConnectAccount.CONNECT_TIMEOUT);
            this.timeouts.push(timeout);

            // Check if connection successful
            web3.eth.net.isListening().then(() => {
                if (this.state.connecting) {
                    // Connection succeeded, set successful.
                    if (!metamask) {
                        this.setState(successfulState);
                    } else {
                        web3.eth.getAccounts((err, accounts) => {
                            if (err || accounts.length == 0) {
                                this.setState(failedState("Could not get account details from MetaMask. Please check MetaMask setup."))
                            } else {
                                this.props.setWeb3(web3, accounts[0]);
                            }
                        });
                    }
                }
            }).catch(() => {
                if (this.state.connecting) {
                    // Connection erred, set failed.
                    this.setState(failedState("Manual connection failed! Please check the blockchain URL."));
                }
            });
        });
    }

    /**
     * Tries to unlock the blockchain account, reverts state and shows error if fails.
     */
    async unlockAccountManual() {
        var failedState = (unlockErr, connectErr) => {
            return {
                web3: (connectErr) ? null : this.state.web3,
                unlocking: false,
                manualConnectError: connectErr,
                manualAccountUnlockError: unlockErr
            };
        };
        this.setState({
            unlocking: true
        }, () => {
            // Set unlock timeout.
            var timeout = setTimeout(() => {
                if (this.state.unlocking) {
                    // Unlocking timed out, set connection failed.
                    this.setState(failedState("", "Manual connection timed out! Please check the blockchain URL."));
                }
            }, ConnectAccount.UNLOCK_TIMEOUT);
            this.timeouts.push(timeout);

            // Check if connection is live
            this.state.web3.eth.net.isListening().then(() => {
                // Unlock the given account
                unlockAccount(this.state.address, this.state.password).then(() => {
                    if (this.state.unlocking) {
                        // Success
                        this.props.setWeb3(this.state.web3, this.state.address);
                    }
                }, () => {
                    if (this.state.unlocking) {
                        // Unlocking erred
                        this.setState(failedState("Account could not be unlocked!\nPlease check the account details.", ""));
                    }
                });
            }, () => {
                if (this.state.unlocking) {
                    // Connection erred
                    this.setState(failedState("", "Manual connection failed! Please check the blockchain URL."));
                }
            });
        });
    }

    /**
     * Handles the inputChange event on the address input.
     * @param event The event.
     */
    handleAddressInputChange(event) {
        event.preventDefault();

        this.setState({
            address: this.addressInput.value
        });
    }

    /**
     * Handles the inputChange event on the password input.
     * @param event The event.
     */ 
    handlePasswordInputChange(event) {
        event.preventDefault();

        this.setState({
            password: this.passwordInput.value
        });
    }

    /**
     * Handles the inputChange event on the URL input.
     * @param event The event.
     */
    handleUrlInputChange(event) {
        event.preventDefault();
        var url = this.urlInput.value;
        if (url == "") {
            url = LOCAL_BLOCKCHAIN_URL;
        }

        this.setState({
            url: url
        });
    }

    /**
     * Handles the keyPress event on the URL input.
     * @param event The event.
     */
    handleUrlInputKeyPress(event) {
        // If the key code is ENTER (13), connect to the blockchain
        if ((event.keyCode ? event.keyCode : event.which) == 13) {
            this.connectManual();
        }
    }

    /**
     * Handles the keyPress event on the address and password inputs.
     * @param event The event.
     */
    handleAccountInputKeyPress(event) {
        // If the key code is ENTER (13), unlock the account
        if ((event.keyCode ? event.keyCode : event.which) == 13) {
            this.unlockAccountManual();
        }
    }
}