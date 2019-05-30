import moment from "moment";
import React from "react";

import Message from "./message.jsx";


/**
 * Component representing a Time Select UI.
 */
export default class TimeSelect extends React.Component {
    /**
     * The CSS block name for this component.
     */
    static blockName = "time-select";

    /**
     * The time input.
     */
    timeInput;

    /**
     * The date input.
     */
    dateInput;

    /**
     * Initialises a new instance of this class.
     * @param props.returnTime Function, into which the selected time is passed.
     */
    constructor(props) {
        super(props);

        this.state = {
            timeErrorMsg: ""
        }
    }

    /**
     * Returns the element representing this component.
     */
    render() {
        return (
            <div className={TimeSelect.blockName + "__container"}>
                <div className={TimeSelect.blockName + "__row-container"}>
                    <div className={TimeSelect.blockName + "__left-container"}>

                        <span className={TimeSelect.blockName + "__select-time-text"}>
                            Select a date and time:
                        </span>

                        <div className={TimeSelect.blockName + "__input-container"}>
                            <input
                                className={TimeSelect.blockName + "__date-input"}
                                ref={r => this.dateInput = r}
                                type="date"
                                onKeyPress={e => this.selectOnEnter(e)}/>

                            <input
                                className={TimeSelect.blockName + "__time-input"}
                                ref={r => this.timeInput = r}
                                type="time"
                                onKeyPress={e => this.selectOnEnter(e)}/>
                        </div>
                    </div>

                    <div className={TimeSelect.blockName + "__right-container"}>
                        <button
                            className={TimeSelect.blockName + "__select-time-button"}
                            onClick={() => this.selectTime()}>
                            Input UNIX Time
                        </button>
                    </div>
                </div>

                {Message.renderError(this.state.timeErrorMsg)}
            </div>
        );
    }

    /**
     * Selects a time.
     */
    selectTime() {
        var date = this.dateInput.value;
        var time = this.timeInput.value;
        if (date == "") {
            this.setState({
                timeErrorMsg: "Please select a date."
            });
            return;
        } else if (time == "") {
            this.setState({
                timeErrorMsg: "Please select a time."
            });
            return;
        }

        this.setState({
            timeErrorMsg: ""
        });
        var dateTime = moment.utc(date + " " + time).unix();

        this.props.returnTime(dateTime);
    }

    /**
     * On enter being pressed, will call selectTime.
     */
    selectOnEnter(event) {
        // If the key code is ENTER (13), select the time
        if ((event.keyCode ? event.keyCode : event.which) == 13) {
            event.preventDefault();
            this.selectTime();
        }
    }
}