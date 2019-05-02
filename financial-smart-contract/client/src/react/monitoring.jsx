import React from "react";

/**
 * The contract monitoring component.
 */
export default class Monitoring extends React.Component {
    /**
     * The CSS block name.
     */
    static blockName = "monitoring";

    /**
     * Initialises a new instance of this class.
     * @param props.web3 The web3 instance.
     * @param props.address The unlocked account address.
     */
    constructor(props) {
        super(props);
    }

    /**
     * Returns the element representing this component.
     */
    render() {
        return (
            <div className={Monitoring.blockName + "__main-container"}>
                
            </div>
        );
    }
}