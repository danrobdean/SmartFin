pragma solidity >=0.4.22 <0.6.0;

// Represents the contract:
// truncate <01/01/2020 00:00:00> and
//       one
//       anytime then
//           truncate <01/02/2020 00:00:00>
//               give scale 2 one
//           truncate <01/03/2020 00:00:00>
//               give scale 3 one
contract LoanVarRepay {
    // A struct representing an anytime sub-contract acquisition time
    struct AnytimeAcquisitionTime {
        // Whether or not the sub-contract has been acquired
        bool acquired;

        // The acquisition time
        uint256 time;
    }

    // Static values
    int256 MAX_INT256 = int256(~(uint256(1) << 255));
    int256 MIN_INT256 = int256(uint256(1) << 255);
    uint256 HORIZON_UNIX_1 = 1577836800;
    uint256 HORIZON_UNIX_2 = 1580515200;
    uint256 HORIZON_UNIX_3 = 1583020800;

    // The contract holder
    address holder;

    // The counter-party
    address counterParty;

    // The stakes of the holder and counter-party
    mapping(address => int256) stakes;

    // Whether or not this contract has been acquired
    bool acquired = false;

    // Whether or not this contract has been fully-updated
    bool fullyUpdated = false;

    // The anytime sub-contract acquisition time
    AnytimeAcquisitionTime subAcquisitionTime = AnytimeAcquisitionTime(false, 0);

    // Constructor, takes the contract holder address
    constructor(address contractHolder) public {
        require(
            contractHolder != msg.sender,
            "Holder and counter-party cannot have the same address."
        );
        // Set the holder and counter-party
        holder = contractHolder;
        counterParty = msg.sender;

        // Initialise stakes to 0
        stakes[counterParty] = 0;
        stakes[holder] = 0;
    }

    // Only allows the holder or counter-party to call a function
    modifier onlyParties() {
        require(
            msg.sender == counterParty || msg.sender == holder,
            "This function can only be called by the holder or the counter-party."
        );

        _;
    }

    // Only allows the holder to call a function
    modifier onlyHolder() {
        require(
            msg.sender == holder,
            "Only the holder may call this function."
        );

        _;
    }

    // Returns the balance of one of the two parties
    function getBalance(bool holderBalance) public view returns (int256) {
        if (holderBalance) {
            return stakes[holder];
        } else {
            return stakes[counterParty];
        }
    }

    // Acquires the anytime sub-contract
    function acquireAnytimeSubContract() public onlyHolder() {
        require(
            !subAcquisitionTime.acquired,
            "The anytime sub-contract has already been acquired."
        );

        subAcquisitionTime.acquired = true;
        subAcquisitionTime.time = now;
        this.update();
    }

    // Acquires this contract
    function acquire() public onlyHolder() {
        require(
            !acquired,
            "This function can only be called before acquisition."
        );
        require(
            now <= HORIZON_UNIX_1,
            "This contract can only be acquired until 01/01/2020 00:00:00 UTC."
        );

        acquired = true;
        transferToHolder(1);
    }

    // Updates the contract balance
    function update() public {
        require(
            acquired,
            "The contract must be acquired before updating."
        );
        require(
            !fullyUpdated,
            "The contract must not be fully-updated when updating."
        );
        require(
            subAcquisitionTime.acquired || now > HORIZON_UNIX_3,
            "The anytime sub-contract must be acquired before updating."
        );

        uint256 repayTime = (subAcquisitionTime.acquired) ? subAcquisitionTime.time : HORIZON_UNIX_3;

        if (repayTime <= HORIZON_UNIX_2) {
            transferToHolder(-2);
        } else {
            transferToHolder(-3);
        }
    }

    // Stake Ether in the contract
    function stake() public payable onlyParties() {
        require(
            uint256(MAX_INT256) >= msg.value,
            "Value being staked is too big to be stored as an int256 value."
        );

        // Update balance
        stakes[msg.sender] = safeAddSigned(stakes[msg.sender], int256(msg.value));
    }

    // Withdraw Ether from the contract
    function withdraw(uint64 amount) public onlyParties() {
        require(
            address(this).balance > 0,
            "Contract does not have enough funds."
        );
        require(
            stakes[msg.sender] > 0,
            "The caller does not have enough stake."
        );

        uint64 finalAmount = amount;

        // Clamp withdrawal amount to total contract balance
        if (address(this).balance < finalAmount) {
            finalAmount = uint64(address(this).balance);
        }

        // Clamp withdrawal amount to party's balance
        if (stakes[msg.sender] < finalAmount) {
            finalAmount = uint64(stakes[msg.sender]);
        }

        // Adjust balance first to prevent re-entrancy bugs
        stakes[msg.sender] = safeSubSigned(stakes[msg.sender], int256(finalAmount));

        // Send Ether (with no gas)
        msg.sender.call.value(finalAmount).gas(0);
    }

    // Transfers the given amount from the holder to the counter-party
    function transferToHolder(int256 amount) private {
        stakes[holder] = safeAddSigned(stakes[holder], amount);
        stakes[counterParty] = safeSubSigned(stakes[counterParty], amount);
    }

    // Add two signed integers if no overflow or underflow can occur
    function safeAddSigned(int256 a, int256 b) private view returns (int256) {
        require(
            (b >= 0 && a <= MAX_INT256 - b) ||
            (b < 0 && a >= MIN_INT256 - b),
            "Integer overflow or underflow."
        );

        return a + b;
    }

    // Subtract one signed integer from another if no overflow or underflow can occur
    function safeSubSigned(int256 a, int256 b) private view returns (int256) {
        require(
            b != MIN_INT256,
            "Integer overflow or underflow."
        );

        return safeAddSigned(a, -b);
    }
}