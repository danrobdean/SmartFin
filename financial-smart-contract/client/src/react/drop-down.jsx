import React from "react";

/**
 * Component which represents a drop-down container.
 */
export default class DropDown extends React.Component {
    /**
     * The CSS block name for this component.
     */
    static blockName = "drop-down";

    /**
     * Initialises a new instance of this class.
     * @param props.title The title.
     * @param props.children The child elements displayed in a drop-down box (optional, if provided a drop-down arrow is displayed).
     */
    constructor(props) {
        super(props);

        this.state = {
            open: false
        };
    }

    /**
     * Returns the element that represents this component.
     */
    render() {
        var containerClassName = DropDown.blockName + "__container";
        var containerClassNames = [
            containerClassName,
            containerClassName + ((this.props.children) ? "--drop-down" : ""),
            this.props.className
        ].join(" ");

        var arrowClassName = DropDown.blockName + "__arrow";
        var arrowClassNames = [
            arrowClassName,
            arrowClassName + ((this.props.children) ? "" : "--hidden"),
            arrowClassName + ((this.state.open) ? "--down" : "--left")
        ].join(" ");

        return (
            <div
                className={containerClassNames}
                onClick={() => this.toggleOpen()}>
                <div className={DropDown.blockName + "__short-container"}>
                    <span className={DropDown.blockName + "__title"}>
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
     * Returns the element representing the drop-down child.
     */
    renderDropDown() {
        if (!this.props.children || !this.state.open) {
            return;
        }

        return (
            <div className={DropDown.blockName + "__drop-down-container"}>
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