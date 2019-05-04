import React from "react";

/**
 * Component representing an animated spinner icon.
 */
export default class Spinner extends React.Component {
    /**
     * The CSS block name for this component.
     */
    static blockName = "spinner";

    /**
     * Renders a waiting notice.
     */
    static renderNotice(text) {
        if (!text) {
            return;
        }

        return (
            <div className={Spinner.blockName + "__notice-container"}>
                <span className={Spinner.blockName + "__notice"}><em>{text}</em></span>
                <Spinner/>
            </div>
        );
    }

    /**
     * Returns the element representing this component.
     */
    render() {
        return <div className={Spinner.blockName}/>;
    }
}