import React from "react";

import Message from "./message.jsx";

import { setObsValue } from "./../js/contract-utils.mjs";

/**
 * Component representing the controls for setting an observable value.
 */
export default class ObsValueControls extends React.Component {
    /**
     * The CSS block name.
     */
    static blockName = "obs-value-controls";

    /**
     * The obserable index input.
     */
    obsIndexInput;

    /**
     * The observable value input.
     */
    obsValueInput;

    /**
     * Initialises a new instance of this class.
     * @param props.address The unlocked account address.
     * @param props.contract The current contract instance.
     * @param props.obsEntries The contract's observable entries.
     * @param props.callback Function to call after setting the observable value.
     */
    constructor(props) {
        super(props);

        var obsEntryMap = this.getObsEntryMap(props.obsEntries);
        this.state = {
            obsEntryMap: obsEntryMap,
            obsIndex: (props.address in obsEntryMap) ? obsEntryMap[props.address][0] : "N/A",
            obsValue: "",
            obsError: "",
            obsErrorDetails: ""
        };
    }

    /**
     * Returns the element representing this component.
     */
    render() {
        var obsIndexOptions = [];
        if (this.props.address in this.state.obsEntryMap) {
            for (var index of this.state.obsEntryMap[this.props.address]) {
                obsIndexOptions.push(
                    <option key={index} value={index}>{index}</option>
                );
            }
        } else {
            obsIndexOptions.push(
                <option key={0} value={null}>N/A</option>
            );
        }

        return (
            <div className={ObsValueControls.blockName + "__set-obs-value-container"}>
                <div className={ObsValueControls.blockName + "__set-obs-value-input-container"}>
                    <div className={ObsValueControls.blockName + "__input-container"}>
                        <span className={ObsValueControls.blockName + "__label"}>
                            Observable Index:
                        </span>
                        <select
                            className={ObsValueControls.blockName + "__obs-index-input"}
                            ref={r => this.obsIndexInput = r}
                            onChange={e => this.handleObsIndexInputChange(e)}>
                            {obsIndexOptions}
                        </select>
                    </div>

                    <div className={ObsValueControls.blockName + "__input-container"}>
                        <span className={ObsValueControls.blockName + "__label"}>
                            Value:
                        </span>

                        <input className={ObsValueControls.blockName + "__obs-value-input"}
                            ref={r => this.obsValueInput = r}
                            onChange={e => this.handleObsValueInputChange(e)}
                            onKeyPress={e => this.handleObsValueInputKeyPress(e)} />
                    </div>
                </div>

                {Message.renderError(this.state.obsError, this.state.obsErrorDetails, ObsValueControls.blockName + "__error")}

                <button
                    onClick={() => this.setObsValue()}>
                    Set Observable Value
                </button>
            </div>
        );
    }

    /**
     * Called when the component receives new props or state.
     * @param prevProps The previous component properties.
     */
    componentDidUpdate(prevProps) {
        // Update the observable entry mapping if the observable entries set has changed.
        if (this.props.obsEntries != prevProps.obsEntries) {
            var obsEntryMap = this.getObsEntryMap(this.props.obsEntries);
            this.setState({
                obsEntryMap: obsEntryMap,
                obsIndex: (this.props.address in obsEntryMap) ? obsEntryMap[this.props.address][0] : "N/A"
            });
        }

        // If contract changes, reset errors
        if (this.props.contract != prevProps.contract) {
            this.setState({
                obsError: "",
                obsErrorDetails: ""
            });
        }
    }

    /**
     * Sets the observable value.
     */
    setObsValue() {
        if (this.state.obsValue === "") {
            this.setState({
                obsError: "Please enter a value."
            });

            return;
        }

        setObsValue(this.props.contract, this.props.address, this.state.obsIndex, this.state.obsValue).then(_ => {
            this.setState({
                obsError: "",
                obsErrorDetails: ""
            });

            this.props.callback();
        }, err => {
            this.setState({
                obsError: "Error occurred while setting the observable value.",
                obsErrorDetails: err.toString()
            })
        });
    }

    /**
     * Gets the observable entry map (maps addresses to the observable indeces they arbitrate).
     */
    getObsEntryMap(obsEntries) {
        var obsEntryMap = {};

        for (var i = 0; i < obsEntries.length; i++) {
            var address = obsEntries[i].getAddress();
            if (!(address in obsEntryMap)) {
                obsEntryMap[address] = [];
            }

            obsEntryMap[address].push(i);
        }

        return obsEntryMap;
    }

    /**
     * Handles the observable index input change event.
     */
    handleObsIndexInputChange(event) {
        event.preventDefault();

        this.setState({
            obsIndex: this.obsIndexInput.value
        });
    }

    /**
     * Handles the observable value input change event.
     */
    handleObsValueInputChange(event) {
        event.preventDefault();

        this.setState({
            obsValue: this.obsValueInput.value
        });
    }

    /**
     * Handles the observable value input key press event.
     */
    handleObsValueInputKeyPress(event) {
        // If the key code is ENTER (13), display deploy modal
        if ((event.keyCode ? event.keyCode : event.which) == 13) {
            event.preventDefault();
            this.setObsValue();
        }
    }
}