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
    obsEntryInput;

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
        var firstIndex = (props.address in obsEntryMap) ? obsEntryMap[props.address].findIndex(elem => !elem.getValue().isDefined()) : -1;
        this.state = {
            obsEntryMap: obsEntryMap,
            obsEntry: (firstIndex >= 0) ? obsEntryMap[props.address][firstIndex] : undefined,
            obsValue: "",
            obsError: "",
            obsErrorDetails: ""
        };
    }

    /**
     * Returns the element representing this component.
     */
    render() {
        var obsEntryOptions = [];
        var seen = [];
        if (this.props.address in this.state.obsEntryMap) {
            for (var entry of this.state.obsEntryMap[this.props.address]) {
                if (!seen.includes(entry.getName()) && !entry.getValue().isDefined()) {
                    obsEntryOptions.push(
                        <option key={entry.getName()} value={entry}>{entry.getName()}</option>
                    );

                    seen.push(entry.getName());
                }
            }
        }

        if (obsEntryOptions.length == 0) {
            obsEntryOptions.push(
                <option key={0} value={null}>N/A</option>
            );
        }

        return (
            <div className={ObsValueControls.blockName + "__set-obs-value-container"}>
                <div className={ObsValueControls.blockName + "__set-obs-value-input-container"}>
                    <div className={ObsValueControls.blockName + "__input-container"}>
                        <span className={ObsValueControls.blockName + "__label"}>
                            Observable Name:
                        </span>
                        <select
                            className={ObsValueControls.blockName + "__obs-index-input"}
                            ref={r => this.obsEntryInput = r}
                            onChange={e => this.handleObsEntryInputChange(e)}>
                            {obsEntryOptions}
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
            var firstIndex = (this.props.address in obsEntryMap) ? obsEntryMap[this.props.address].findIndex(elem => !elem.getValue().isDefined()) : -1;
            this.setState({
                obsEntryMap: obsEntryMap,
                obsEntry: (firstIndex >= 0) ? obsEntryMap[this.props.address][firstIndex] : undefined
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
        if (!this.props.address in this.state.obsEntryMap) {
            this.setState({
                obsError: "This account is not the arbiter of any observables."
            });

            return;
        }

        if (!this.state.obsValue || isNaN(this.state.obsValue)) {
            this.setState({
                obsError: "Please enter a numeric value."
            });

            return;
        }

        var eligibleObsEntries = this.state.obsEntryMap[this.props.address].filter(entry => entry.getName() == this.state.obsEntry.getName());
        for (var entry of eligibleObsEntries) {
            setObsValue(this.props.contract, this.props.address, entry.getIndex(), this.state.obsValue).then(_ => {
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

            obsEntryMap[address].push(obsEntries[i]);
        }

        return obsEntryMap;
    }

    /**
     * Handles the observable index input change event.
     */
    handleObsEntryInputChange(event) {
        event.preventDefault();

        this.setState({
            obsEntry: this.obsEntryInput.value
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