import React from "react";

import { splitContract } from "./../js/contract-utils.mjs";

/**
 * A component which displays a SmartFin contract, with a certain combinator highlighted red, and all previous combinators highlighted green.
 */
export default class ContractText extends React.Component {
    /**
     * The CSS block name.
     */
    static blockName = "contract-text";

    /**
     * Returns the element representing this component.
     */
    render() {
        var contract = splitContract(this.props.contract);
        var spans = [];

        // Add the green highlighted text
        if (this.props.highlightIndex !== -1) {
            let completeText = contract.slice(0, this.props.highlightIndex).join(" ");
            if (this.props.highlightIndex !== undefined && this.props.highlightIndex < contract.length) {
                completeText += " ";
            }

            spans.push(
                <span key={spans.length} className={ContractText.blockName + "__contract-complete"}>
                    {completeText}
                </span>
            );
        }

        // Add the red highlighted text and normal text
        if (this.props.highlightIndex !== undefined && this.props.highlightIndex < contract.length) {
            let incompleteText = contract.slice(this.props.highlightIndex + 1).join(" ");

            if (this.props.highlightIndex >= 0) {
                spans.push(
                    <span key={spans.length} className={ContractText.blockName + "__contract-highlighted"}>
                        {contract[this.props.highlightIndex]}
                    </span>
                );

                incompleteText = " " + incompleteText;
            }

            spans.push(
                <span key={spans.length} className={ContractText.blockName + "__contract-incomplete"}>
                    {incompleteText}
                </span>
            );
        }

        return (
            <div className={ContractText.blockName + "__container"}>
                {spans}
            </div>
        );
    }
}