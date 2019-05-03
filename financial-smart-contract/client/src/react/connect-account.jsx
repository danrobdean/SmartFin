import React from "react";

import { setupWeb3, unlockAccount } from "./../js/contract-utils.mjs";

const LOCAL_BLOCKCHAIN_URL = "http://localhost:8545";

// The UI for connecting an account via MetaMask or direct input.
export default class ConnectAccount extends React.Component {
    // The CSS block name for this component
    static blockName = "connect-account";

    // Reference to the manual account address input.
    addressInput;

    // Reference to the manual account password input.
    passwordInput;

    // Reference to the manual blockchain URL input.
    urlInput;

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
            web3: null
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
                        </div>
                    </div>
                </div>
            </div>
        );
    }

    /**
     * Attempts to connect to the MetaMask instance.
     */
    async connectMetaMask() {
        alert("Not implemented.");
    }

    /**
     * Attempts to connect to the manually input blockchain.
     */
    async connectManual() {
        // Setup web3 connection
        var web3 = setupWeb3(this.state.url);

        // Check if connection successful
        web3.eth.net.isListening().then(() => {
            this.setState({
                web3: web3
            });
        }, () => {
            this.setState({
                web3: null
            });
            alert("Manual connection failed!\nPlease check the blockchain URL.");
        });
    }

    async unlockAccountManual() {
        // Check if connection is live
        this.state.web3.eth.net.isListening().then(() => {
            // Unlock the given account
            unlockAccount(this.state.address, this.state.password).then(() => {
                this.props.setWeb3(this.state.web3, this.state.address);
            }, () => {
                alert("Account could not be unlocked!\nPlease check the account details.");
            });
        }, () => {
            this.setState({
                web3: null
            });

            alert("Manual connection failed!\nPlease check the blockchain URL.");
        })
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