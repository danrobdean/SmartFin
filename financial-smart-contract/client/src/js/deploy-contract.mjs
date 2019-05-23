import readline from "readline";
import { unlockDefaultAccount, loadAndDeployContract, serializeCombinatorContract } from "./contract-utils.mjs";

// Setup readline
const r1 = readline.createInterface({
    input: process.stdin,
    output: process.stdout
});

// Obtain contract from IO, then handle
r1.question("Please input a combinator contract, or press ENTER to exit: ", (answer) => {
    var combinatorContract = answer.trim();

    // Close if no contract entered
    if (combinatorContract == "") {
        r1.close();
        return;
    }

    r1.question("Please input the address of the holder, or press ENTER to exit: ", (answer) => {
        var holder = answer.trim();

        // Close if no holder entered
        if (holder == "") {
            r1.close();
            return;
        }

        // Set default account for transactions (this is pre-defined on our testing blockchain) and unlock
        var sender = unlockDefaultAccount();
    
        // Serialize contract
        var serializedCombinatorContract = serializeCombinatorContract(combinatorContract);

        r1.question("Would you like the contract to spend gas upon withdrawal? Y/N: ", (answer) => {
            var useGas = (answer === "Y");

            // Deploy contract
            loadAndDeployContract(serializedCombinatorContract, holder, sender, useGas).then((_) => {}, err => console.error(err));
            r1.close();
        });
    });
});