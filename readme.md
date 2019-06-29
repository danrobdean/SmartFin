# SmartFin - Implementing a Financial Domain-Specific Language for Smart Contracts

This is the repository for the *SmartFin - Implementing a Financial Domain-Specific Language for Smart Contracts* project, which can be accessed on GitHub at https://github.com/danrobdean/SmartFin. In this project, we provide a smart contract implementation for a financial contract domain-specific language, SmartFin. SmartFin is derived from a DSL presented by Simon Peyton Jones, Jean-Marc Eber, and Julian Seward in *Composing Contracts:
An Adventure in Financial Engineering*. We also present a web client for writing and evaluating SmartFin financial contracts, and managing their smart contract representations.

The `financial-smart-contract` folder contains the main deliverables of the project.

The `report-solidity-contracts` folder contains several Solidity implementations of SmartFin financial contract and a code snippet showcasing a reentrancy vulnerability, used for the report's evaluation and introduction respectively.

The report log is also included in this folder, as well as the results of several gas-cost comparisons for the case studies used in the evaluation section of the report.

The `report` folder contains the final report produced for this project in PDF form, as well as the LaTeX source files and images.

The `user-manual` folder contains the user manual for the web client in PDF form, as well as the LaTeX source files and images.

The `presentation` folder contains the final presentation produced for this project, as a PPTX file with notes.

### `financial-smart-contract`

This folder contains several folders, scripts, the settings for a local development blockchain, and a readme which explains running these scripts, and running the client build.

The `client` folder contains the source files, tests, and development/build setup files for the web client.

The `contract` folder contains the source files, tests, and development/build setup files for the smart contract which takes a SmartFin financial contract and represents it.

The `dist-client` folder contains the distributables, i.e. a script which runs a server to serve the web client, and the web client build.
