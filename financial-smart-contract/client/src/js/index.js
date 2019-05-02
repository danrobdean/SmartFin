import "@babel/polyfill";

import React from "react";
import ReactDOM from "react-dom";

import Main from "./../react/main.jsx";
import "./../css/style.css";

window.onload = () => ReactDOM.render(<Main />, document.getElementById("root"));