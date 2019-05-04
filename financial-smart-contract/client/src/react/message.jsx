import React from "react";

/**
 * The component representing an Message.
 */
export default class Message extends React.Component {
    /**
     * The CSS block name for this component.
     */
    static blockName = "message";

    /**
     * Initialises a new instance of this class.
     * @param props.title The title.
     * @param props.children The long description (optional, if provided a dropdown arrow is displayed).
     */
    constructor(props) {
        super(props);

        this.state = {
            open: false
        };
    }

    /**
     * Renders an error message.
     * @param error The error message.
     * @param details The error details (optional).
     */
    static renderError(error, details=null) {
        return Message.renderMessage("error", error, details);
    }

    /**
     * Renders an info message.
     * @param info The info message.
     * @param details The info details (optional).
     */
    static renderInfo(info, details=null) {
        return Message.renderMessage("info", info, details);
    }

    /**
     * Renders a warning message.
     * @param warning The warning message.
     * @param details The warning details (optional).
     */
    static renderWarning(warning, details=null) {
        return Message.renderMessage("warning", (warning) ? "Warning: " + warning : warning, details);
    }

    static renderMessage(type, title, details) {
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

        return <Message title={title} type={type}>{details}</Message>;
    }

    /**
     * Returns the element that represents this component.
     */
    render() {
        var containerClassName = Message.blockName + "__container";
        var containerClassNames = [
            containerClassName,
            containerClassName + "--" + this.props.type,
            containerClassName + ((this.props.children) ? "--dropdown" : "")
        ].join(" ");

        var arrowClassName = Message.blockName + "__arrow";
        var arrowClassNames = [
            arrowClassName,
            arrowClassName + ((this.props.children) ? "" : "--hidden"),
            arrowClassName + ((this.state.open) ? "--down" : "--left"),
            arrowClassName + "--" + this.props.type
        ].join(" ");

        return (
            <div
                className={containerClassNames}
                onClick={() => this.toggleOpen()}>
                <div className={Message.blockName + "__short-container"}>
                    <span className={Message.blockName + "__title"}>
                        {this.props.title}
                    </span>
                    <i
                        className={arrowClassNames}
                        onClick={() => this.toggleOpen()}/>
                </div>
                {this.renderDropDown()}
            </div>
        );
    }

    /**
     * Returns the element representing the dropdown description.
     */
    renderDropDown() {
        if (!this.props.children || !this.state.open) {
            return;
        }

        var dropdownClassName = Message.blockName + "__dropdown-container";
        var dropdownClassNames = [
            dropdownClassName,
            dropdownClassName + "--" + this.props.type
        ].join(" ");

        return (
            <div className={dropdownClassNames}>
                {this.props.children}
            </div>
        );
    }

    /**
     * Toggles whether or not the drop-down description is open or closed.
     */
    toggleOpen() {
        this.setState({
            open: !this.state.open
        });
    }
}