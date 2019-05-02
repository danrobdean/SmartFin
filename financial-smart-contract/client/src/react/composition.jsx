import React from "react";

/**
 * The contract composition component.
 */
export default class Composition extends React.Component {
    /**
     * The CSS block name.
     */
    static blockName = "composition";

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
            <div className={Composition.blockName + "__main-container"}>

            </div>
        );
    }
}