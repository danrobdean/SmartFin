import React from "react";

/**
 * The main menu component.
 */
export default class MainMenu extends React.Component {
    /**
     * The CSS block name
     */
    static blockName = "main-menu";

    /**
     * Initialises a new instance of this class.
     * @param props.goToComposition Function to transition to the contract composition menu.
     * @param props.goToMonitoring Function to transition to the contract monitoring menu.
     */
    constructor(props) {
        super(props);
    }

    /**
     * Returns the element representing this class.
     */
    render() {
        return (
            <div className={MainMenu.blockName + "__main-container"}>
                <div className={MainMenu.blockName + "__button-container"}>
                    <button 
                        className={MainMenu.blockName + "__composition-button"}
                        onClick={() => this.props.goToComposition()}>
                        Compose a new contract
                    </button>
                    <button 
                        className={MainMenu.blockName + "__monitoring-button"}
                        onClick={() => this.props.goToMonitoring()}>
                        Monitor an existing contract
                    </button>
                </div>
            </div>
        );
    }
}