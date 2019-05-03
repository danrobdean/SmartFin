import React from "react";

/**
 * The Modal component.
 */
export default class Modal extends React.Component {
    /**
     * The CSS block name for this component.
     */
    static blockName = "modal";

    /**
     * Returns the element representing this component.
     */
    render() {
        var visibilityModifier = this.props.visible ? "" : "--hidden";

        return (
            <div
                className={Modal.blockName + "__overlay" + visibilityModifier}
                onClick={() => this.props.closeModal()}>
                <div
                    className={Modal.blockName + "__container"}
                    onClick={e => e.stopPropagation()}>
                    <div className={Modal.blockName + "__titlebar"}>
                        <h2 className={Modal.blockName + "__title"}>
                            {this.props.title}
                        </h2>
                        <div
                            className={Modal.blockName + "__close-button"}
                            onClick={() => this.props.closeModal()}>
                            <h3 className={Modal.blockName + "__close-icon"}>&times;</h3>
                        </div>
                    </div>
                    <div className={Modal.blockName + "__content-container"}>
                        {this.props.children}
                    </div>
                </div>
            </div>
        );
    }
}