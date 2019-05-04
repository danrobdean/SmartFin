import React from "react";

/**
 * The Help component.
 */
export default class Help extends React.Component {
    /**
     * The CSS block name for this component.
     */
    static blockName = "help";

    /**
     * Returns the element that represents this component.
     */
    render() {
        return (
            <span className={Help.blockName + "__text"}>
                First, write a contract in the text box.
                <br/>
                <br/>
                If the contract concerns time, press the <em>Select Time</em> button to input a time into the contract at the cursor's position.
                <br/>
                <br/>
                Once the contract is complete, press the <em>Submit Contract</em> button to deploy it to the blockchain.
            </span>
        )
    }
}