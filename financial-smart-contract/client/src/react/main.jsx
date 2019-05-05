import React from "react";

import Composition from "./composition.jsx";
import ConnectAccount from "./connect-account.jsx";
import MainMenu from "./main-menu.jsx";
import Monitoring from "./monitoring.jsx";

/** 
 * The main UI component.
 */
export default class Main extends React.Component {
    /**
     * The CSS block name for this component.
     */
    static blockName = "main";

    /**
     * Initialises a new instance of this class.
     * @param props The component properties.
     */
    constructor(props) {
        super(props);

        this.state = {
            web3: null,
            address: "",
            appState: "connect",
            contract: null
        }
    }

    /**
     * Returns the element representing this component.
     */
    render() {
        switch (this.state.appState) {
            case "connect": {
                return (
                    <ConnectAccount
                        setWeb3={(web3, address) => this.setWeb3Instance(web3, address)}/>
                );
            }
            case "main-menu": {
                return (
                    <MainMenu
                        goToComposition={() => this.goToComposition()}
                        goToMonitoring={() => this.goToMonitoring()}/>
                );
            }
            case "composition": {
                return this.renderWithBackButton(
                    <Composition
                        web3={this.state.web3}
                        address={this.state.address}
                        setContract={contract => this.setContract(contract)}/>
                );
            }
            case "monitoring": {
                return this.renderWithBackButton(
                    <Monitoring
                        web3={this.state.web3}
                        address={this.state.address}
                        setContract={contract => this.setContract(contract)}
                        contract={this.state.contract}/>
                    );
            }
            default: return <div />;
        }
    }

    /**
     * Renders the given component in a container with a button which returns to the main menu.
     * @param component The component to render with a back button.
     */
    renderWithBackButton(component) {
        return (
            <div className={Main.blockName + "__back-button-container"}>
                {/* The title bar and back button */}
                <div className={Main.blockName + "__wrapper"}>
                    <button
                        className={Main.blockName + "__back-button"}
                        onClick={() => this.goToMainMenu()}>
                        {"\u2190"}
                    </button>
                </div>

                {/* The component */}
                <div className={Main.blockName + "__container"}>
                    {component}
                </div>
            </div>
        );
    }

    /**
     * Set the web3 instance.
     * @param web3 The web3 instance
     */
    setWeb3Instance(web3, address) {
        this.setState({
            web3: web3,
            address: address
        });

        this.goToMainMenu();
    }

    /**
     * Set the contract instance.
     * @param contract The contract instance.
     */
    setContract(contract) {
        this.setState({
            contract: contract
        });
    }

    /**
     * Displays the main menu.
     */
    goToMainMenu() {
        this.setState({
            appState: "main-menu"
        });
    }

    /**
     * Displays the contract composition menu.
     */
    goToComposition() {
        this.setState({
            appState: "composition"
        });
    }

    /**
     * Displays the contract monitoring menu.
     */
    goToMonitoring() {
        this.setState({
            appState: "monitoring"
        });
    }
}