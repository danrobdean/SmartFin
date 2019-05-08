import React from "react";

import DropDown from "./drop-down.jsx";

/**
 * The component representing an Message.
 */
export default class Message extends React.Component {
    /**
     * The CSS block name for this component.
     */
    static blockName = "message";

    /**
     * Renders an error message.
     * @param error The error message.
     * @param details The error details (optional).
     * @param className Extra class name (optional).
     */
    static renderError(error, details=null, className="") {
        return Message.renderMessage("error", error, details, className);
    }

    /**
     * Renders a success message.
     * @param success The success message.
     * @param details The success details (optional).
     * @param className Extra class name (optional).
     */
    static renderSuccess(success, details=null, className="") {
        return Message.renderMessage("success", success, details, className);
    }

    /**
     * Renders an info message.
     * @param info The info message.
     * @param details The info details (optional).
     * @param className Extra class name (optional).
     */
    static renderInfo(info, details=null, className="") {
        return Message.renderMessage("info", info, details, className);
    }

    /**
     * Renders a warning message.
     * @param warning The warning message.
     * @param details The warning details (optional).
     * @param className Extra class name (optional).
     */
    static renderWarning(warning, details=null, className="") {
        return Message.renderMessage("warning", (warning) ? "Warning: " + warning : warning, details, className);
    }

    static renderMessage(type, title, details, className) {
        if (!title) {
            return;
        }

        // If details is an array, add line breaks between elements
        if (Array.isArray(details)) {
            var i = 0;
            const addBreaks = (acc, cur) => {
                i++;
                return acc.concat(cur, <br key={i}/>);
            }

            details = details.reduce(addBreaks, []);
        }

        return <Message className={className} title={title} type={type}>{details}</Message>;
    }

    /**
     * Returns the element that represents this component.
     */
    render() {
        var containerClassName = Message.blockName + "__container";
        var containerClassNames = [
            containerClassName,
            containerClassName + "--" + this.props.type,
            this.props.className
        ].join(" ");

        return (
            <div
                className={containerClassNames}>
                <DropDown title={this.props.title}>
                    {this.props.children}
                </DropDown>
            </div>
        );
    }
}