extern crate pwasm_std;
extern crate pwasm_test;
mod common;

#[allow(unused_imports)]
use self::pwasm_std::{ vec, types::{ Address, U256 } };
use self::pwasm_test::ext_reset;
use self::common::{ setup_contract, serialize_32_bit_int, serialize_signed_64_bit_int, FinancialScContract, FinancialScInterface };

// The value of the contract is based on the given serialized combinator vector
#[test]
fn correct_value_zero() {
    let mut contract = setup_contract(vec![0]).contract;

    // Check that the value is correct
    assert_eq!(contract.get_value(), 0);
}

// The value of the contract is based on the given serialized combinator vector
#[test]
fn correct_value_one() {
    let mut contract = setup_contract(vec![1]).contract;

    // Check that the value is correct
    assert_eq!(contract.get_value(), 1);
}

// The value of the contract is based on the given serialized combinator vector
#[test]
fn correct_value_and() {
    let mut contract = setup_contract(vec![2, 1, 1]).contract;

    // Check that the value is correct
    assert_eq!(contract.get_value(), 2);
}

// The value of the or combinator is correct given a left or choice
#[test]
fn correct_value_or_left() {
    let mut contract = setup_contract(vec![3, 0, 1]).contract;
    
    // Set the or choice and check the value
    contract.set_or_choice(0, true);
    assert_eq!(contract.get_value(), 0);
}

// The value of the or combinator is correct given a right or choice
#[test]
fn correct_value_or_right() {
    let mut contract = setup_contract(vec![3, 0, 1]).contract;
    
    // Set the or choice and check the value
    contract.set_or_choice(0, false);
    assert_eq!(contract.get_value(), 1);
}

// The value of an expired truncated contract is 0
#[test]
fn expired_truncate_worth_0() {
    // Create contract truncate 0 one
    let mut timestamp = serialize_32_bit_int(0).to_vec();
    let mut combinator_contract = vec![4];
    combinator_contract.append(&mut timestamp);
    combinator_contract.append(&mut vec![1]);
    let mut contract = setup_contract(combinator_contract).contract;

    // Check that contract value is 0 at timestamp 1
    ext_reset(|e| e.timestamp(1));
    assert_eq!(contract.get_value(), 0);
}

// The value of a non-expired truncated contract is correct
#[test]
fn non_expired_truncate_value_correct() {
    // Create contract truncate 1 one
    let mut timestamp = serialize_32_bit_int(1).to_vec();
    let mut combinator_contract = vec![4];
    combinator_contract.append(&mut timestamp);
    combinator_contract.append(&mut vec![1]);
    let mut contract = setup_contract(combinator_contract).contract;

    // Check that contract value is 1 at timestamp 0
    ext_reset(|e| e.timestamp(0));
    assert_eq!(contract.get_value(), 1);
}

// The value of and with one expired sub-contract is correct
#[test]
fn expired_and_correct() {
    // Create contract and truncate 0 one one
    let mut timestamp = serialize_32_bit_int(0).to_vec();
    let mut combinator_contract = vec![3, 4];
    combinator_contract.append(&mut timestamp);
    combinator_contract.append(&mut vec![1, 1]);
    let mut contract = setup_contract(combinator_contract).contract;

    // Check that contract value is 1 at timestamp 1
    ext_reset(|e| e.timestamp(1));
    assert_eq!(contract.get_value(), 1);
}

// The value of or with one expired sub-contract is correct
#[test]
fn expired_or_correct() {
    // Create contract or truncate 0 one zero
    let mut timestamp = serialize_32_bit_int(0).to_vec();
    let mut combinator_contract = vec![3, 4];
    combinator_contract.append(&mut timestamp);
    combinator_contract.append(&mut vec![1, 0]);
    let mut contract = setup_contract(combinator_contract).contract;

    // Check that contract value is 0 at timestamp 1 with no or-choice
    ext_reset(|e| e.timestamp(1));
    assert_eq!(contract.get_value(), 0);
}

// The value of or with one expired sub-contract and a conflicting or-choice is correct
#[test]
fn expired_or_ignores_choice() {
    // Create contract or truncate 0 one zero
    let mut timestamp = serialize_32_bit_int(0).to_vec();
    let mut combinator_contract = vec![3, 4];
    combinator_contract.append(&mut timestamp);
    combinator_contract.append(&mut vec![1, 0]);
    let mut contract = setup_contract(combinator_contract).contract;

    // Check that contract value is 0 at timestamp 1 with left or-choice
    contract.set_or_choice(0, true);
    ext_reset(|e| e.timestamp(1));
    assert_eq!(contract.get_value(), 0);
}

// The value of a scale combinator with the scale value provided is correct
#[test]
fn scale_with_provided_scale_value_has_correct_value() {
    // Create contract or scale 2 one
    let mut scale_value = serialize_signed_64_bit_int(2).to_vec();
    let mut combinator_contract = vec![5, 1];
    combinator_contract.append(&mut scale_value);
    combinator_contract.append(&mut vec![1]);
    let mut contract = setup_contract(combinator_contract).contract;

    // Check that contract value is 2
    assert_eq!(contract.get_value(), 2);
}

// The value of a scale combinator with a negative scale value provided is correct
#[test]
fn scale_with_provided_negative_scale_value_has_correct_value() {
    // Create contract or scale -1 one
    let mut scale_value = serialize_signed_64_bit_int(-2).to_vec();
    let mut combinator_contract = vec![5, 1];
    combinator_contract.append(&mut scale_value);
    combinator_contract.append(&mut vec![1]);
    let mut contract = setup_contract(combinator_contract).contract;

    // Check that contract value is -2
    assert_eq!(contract.get_value(), -2);
}

// The value of a scale combinator with an agreed-upon observable scale value is correct
#[test]
fn scale_with_concrete_obs_value_has_correct_value() {
    // Create contract or scale obs one
    let mut contract_details = setup_contract(vec![5, 0, 1]);

    // Propose obs_value_0 = 2 from both parties
    ext_reset(|e| e.sender(contract_details.holder));
    contract_details.contract.propose_obs_value(0, 2);

    ext_reset(|e| e.sender(contract_details.counter_party));
    contract_details.contract.propose_obs_value(0, 2);

    // Check that contract value is 2
    assert_eq!(contract_details.contract.get_value(), 2);
}

// The value of a scale combinator with an agreed-upon negative observable scale value is correct
#[test]
fn scale_with_concrete_negative_obs_value_has_correct_value() {
    // Create contract or scale obs one
    let mut contract_details = setup_contract(vec![5, 0, 1]);

    // Propose obs_value_0 = -2 from both parties
    ext_reset(|e| e.sender(contract_details.holder));
    contract_details.contract.propose_obs_value(0, -2);

    ext_reset(|e| e.sender(contract_details.counter_party));
    contract_details.contract.propose_obs_value(0, -2);

    // Check that contract value is -2
    assert_eq!(contract_details.contract.get_value(), -2);
}

// The value of a scale combinator with an agreed-upon observable scale value does not change after one extra proposal
#[test]
fn scale_with_concrete_obs_value_has_correct_value_after_extra_proposal() {
    // Create contract or scale obs one
    let mut contract_details = setup_contract(vec![5, 0, 1]);

    // Propose obs_value_0 = 2 from both parties
    ext_reset(|e| e.sender(contract_details.holder));
    contract_details.contract.propose_obs_value(0, 2);

    ext_reset(|e| e.sender(contract_details.counter_party));
    contract_details.contract.propose_obs_value(0, 2);

    // Check that contract value is 2
    assert_eq!(contract_details.contract.get_value(), 2);

    // Propose obs_value_0 = 3 from the counter-party
    contract_details.contract.propose_obs_value(0, 3);

    // Check that contract value is still 2
    assert_eq!(contract_details.contract.get_value(), 2);
}

// The value of a scale combinator with an agreed-upon observable scale value changes after a new agreed-upon proposal
#[test]
fn scale_with_concrete_obs_value_has_correct_value_after_second_agreement() {
    // Create contract or scale obs one
    let mut contract_details = setup_contract(vec![5, 0, 1]);

    // Propose obs_value_0 = 2 from both parties
    ext_reset(|e| e.sender(contract_details.holder));
    contract_details.contract.propose_obs_value(0, 2);

    ext_reset(|e| e.sender(contract_details.counter_party));
    contract_details.contract.propose_obs_value(0, 2);

    // Check that contract value is 2
    assert_eq!(contract_details.contract.get_value(), 2);

    // Propose obs_value_0 = 3 from both parties
    contract_details.contract.propose_obs_value(0, 3);

    ext_reset(|e| e.sender(contract_details.holder));
    contract_details.contract.propose_obs_value(0, 3);

    // Check that contract value is now 3
    assert_eq!(contract_details.contract.get_value(), 3);
}

// The value of a give contract is correct
#[test]
fn give_value_correct() {
    // Create contract give one
    let mut contract = setup_contract(vec![6, 1]).contract;

    // Check that the contract value is -1
    ext_reset(|e| e.timestamp(0));
    assert_eq!(contract.get_value(), -1);
}

// The value of a then contract is the value of the first sub-combinator pre-expiry
#[test]
fn then_value_equals_first_sub_combinator_pre_expiry() {
    // Create contract then truncate 1 one zero
    let mut timestamp = serialize_32_bit_int(1).to_vec();
    let mut contract_definiton = vec![7, 4];
    contract_definiton.append(&mut timestamp);
    contract_definiton.append(&mut vec![1, 0]);

    let mut contract = setup_contract(contract_definiton).contract;

    // Check value equals 1
    ext_reset(|e| e.timestamp(1));
    assert_eq!(contract.get_value(), 1);
}

// The value of a then contract is the value of the second sub-combinator post-expiry
#[test]
fn then_value_equals_second_sub_combinator_post_expiry() {
    // Create contract then truncate 1 one zero
    let mut timestamp = serialize_32_bit_int(1).to_vec();
    let mut contract_definiton = vec![7, 4];
    contract_definiton.append(&mut timestamp);
    contract_definiton.append(&mut vec![1, 0]);

    let mut contract = setup_contract(contract_definiton).contract;

    // Check value equals 0
    ext_reset(|e| e.timestamp(2));
    assert_eq!(contract.get_value(), 0);
}

// Attempting to get the value of a contract before calling the constructor is not allowed
#[test]
#[should_panic]
fn should_panic_if_getting_value_before_initialised() {
    let mut contract = FinancialScContract::new();
    contract.get_value();
}

// Evaluating a contract with an ambiguous or choice is not allowed
#[test]
#[should_panic]
fn should_panic_if_ambiguous_or_choice() {
    let mut contract = setup_contract(vec![3, 1, 0]).contract;
    contract.get_value();
}

// Getting the value of a contract with an undefined observable is not allowed
#[test]
#[should_panic]
fn should_panic_if_getting_value_with_undefined_observable() {
    let mut contract = setup_contract(vec![5, 0, 1]).contract;
    contract.get_value();
}

// Getting the value of a contract with an observable without a concrete value is not allowed
#[test]
#[should_panic]
fn should_panic_if_getting_value_with_observable_without_concrete_value() {
    let mut contract_details = setup_contract(vec![5, 0, 1]);

    // Propose two different values
    ext_reset(|e| e.sender(contract_details.holder));
    contract_details.contract.propose_obs_value(0, 1);

    ext_reset(|e| e.sender(contract_details.counter_party));
    contract_details.contract.propose_obs_value(0, 2);
    contract_details.contract.get_value();
}