import React from "react";

/**
 * The drop-down description component.
 */
export default class Description extends React.Component {
    /**
     * The CSS block name.
     */
    static blockName = "description";

    /**
     * Initialises a new instance of this class.
     * @param props.short The short description.
     * @param props.long The long description.
     */
    constructor(props) {
        super(props);

        this.state = {
            open: false
        };
    }

    /**
     * Returns the element representing this component.
     */
    render() {
        var extendedModifier = open ? "" : "--hidden";
        var openModifier = open ? "--down" : "";

        return (
            <div
                className={blockName + "__container"}
                onClick={() => this.handleClick()}>
                <div className={blockName + "__short-container"}>
                    <span className={blockName + "__short-desc"}>{this.props.short}</span>
                    <span className={blockName + "__open" + openModifier}>{"<"}</span>
                </div>
                <div className={blockName + "__extended" + extendedModifier}>
                    <span className={blockName + "__long-desc"}>{this.props.long}</span>
                </div>
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