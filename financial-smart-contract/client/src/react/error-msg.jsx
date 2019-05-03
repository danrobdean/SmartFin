import React from "react";

/**
 * The component representing an Error Message.
 */
export default class ErrorMsg extends React.Component {
    /**
     * The CSS block name for this component.
     */
    static blockName = "error-msg";

    /**
     * Returns the element that represents this component.
     */
    render() {
        var visibilityModifier = this.props.error ? "" : "--hidden";

        return (
            <div className={ErrorMsg.blockName + "__container" + visibilityModifier}>
                <span className={ErrorMsg.blockName + "__message"}>
                    {this.props.children}
                </span>
            </div>
        );
    }
}