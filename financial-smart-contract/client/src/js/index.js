import "@babel/polyfill";

import React from "react";
import ReactDOM from "react-dom";

import Main from "./../react/main.jsx";
import "./../scss/style.scss";

window.onload = () => ReactDOM.render(<Main />, document.getElementById("root"));