extern crate pwasm_std;
extern crate pwasm_test;
mod common;

#[allow(unused_imports)]
use self::pwasm_std::{ vec, types::{ Address, U256 } };
use self::pwasm_test::{ ext_update };
use self::common::{ setup_contract, FinancialScContract, FinancialScInterface, address_to_i64 };

// The value of the contract is based on the given serialized combinator vector
#[test]
fn correct_value_zero() {
    let mut contract_details = setup_contract(vec![0]);

    ext_update(|e| e.sender(contract_details.holder));
    contract_details.contract.acquire();

    // Check that the value is correct
    assert_eq!(contract_details.contract.get_balance(true), 0);
}

// The value of the contract is based on the given serialized combinator vector
#[test]
fn correct_value_one() {
    let mut contract_details = setup_contract(vec![1]);

    ext_update(|e| e.sender(contract_details.holder));
    contract_details.contract.acquire();

    // Check that the value is correct
    assert_eq!(contract_details.contract.get_balance(true), 1);
}

// The value of the contract is based on the given serialized combinator vector
#[test]
fn correct_value_and() {
    let mut contract_details = setup_contract(vec![2, 1, 1]);

    ext_update(|e| e.sender(contract_details.holder));
    contract_details.contract.acquire();

    // Check that the value is correct
    assert_eq!(contract_details.contract.get_balance(true), 2);
}

// The value of the or combinator is correct given a left or choice
#[test]
fn correct_value_or_left() {
    let mut contract_details = setup_contract(vec![3, 0, 1]);

    // Set the or choice and check the value
    ext_update(|e| e.sender(contract_details.holder));
    contract_details.contract.set_or_choice(0, true);
    contract_details.contract.acquire();

    // Check that the value is correct    
    assert_eq!(contract_details.contract.get_balance(true), 0);
}

// The value of the or combinator is correct given a right or choice
#[test]
fn correct_value_or_right() {
    let mut contract_details = setup_contract(vec![3, 0, 1]);

    // Set the or choice and check the value
    ext_update(|e| e.sender(contract_details.holder));
    contract_details.contract.set_or_choice(0, false);
    contract_details.contract.acquire();

    // Check that the value is correct    
    assert_eq!(contract_details.contract.get_balance(true), 1);
}

// The value of a non-expired truncated contract is correct
#[test]
fn non_expired_truncate_value_correct() {
    // Create contract truncate 0 one
    let mut contract_details = setup_contract(vec![4, 0, 1]);

    // Check that contract value is 0 at timestamp 1
    ext_update(|e| e
        .timestamp(0)
        .sender(contract_details.holder)
    );
    contract_details.contract.acquire();

    assert_eq!(contract_details.contract.get_balance(true), 1);
}

// The value of and with one expired sub-contract is correct
#[test]
fn expired_and_correct() {
    // Create contract and truncate 0 one one
    let mut contract_details = setup_contract(vec![3, 4, 0, 1, 1]);

    // Check that contract value is 1 at timestamp 1
    ext_update(|e| e
        .timestamp(1)
        .sender(contract_details.holder)
    );
    contract_details.contract.acquire();

    assert_eq!(contract_details.contract.get_balance(true), 1);
}

// The value of or with one expired sub-contract is correct
#[test]
fn expired_or_correct() {
    // Create contract or truncate 0 one zero
    let mut contract_details = setup_contract(vec![3, 4, 0, 1, 0]);

    // Check that contract value is 0 at timestamp 1 with no or-choice
    ext_update(|e| e
        .timestamp(1)
        .sender(contract_details.holder)
    );
    contract_details.contract.acquire();

    assert_eq!(contract_details.contract.get_balance(true), 0);
}

// The value of or with one expired sub-contract and a conflicting or-choice is correct
#[test]
fn expired_or_ignores_choice() {
    // Create contract or truncate 0 one zero
    let mut contract_details = setup_contract(vec![3, 4, 0, 1, 0]);

    // Check that contract value is 0 at timestamp 1 with left or-choice
    ext_update(|e| e
        .timestamp(1)
        .sender(contract_details.holder)
    );
    contract_details.contract.set_or_choice(0, true);
    contract_details.contract.acquire();

    assert_eq!(contract_details.contract.get_balance(true), 0);
}

// The value of a scale combinator with the scale value provided is correct
#[test]
fn scale_with_provided_scale_value_has_correct_value() {
    // Create contract or scale 2 one
    let scale = 2;
    let mut contract_details = setup_contract(vec![5, 1, scale, 1]);

    ext_update(|e| e.sender(contract_details.holder));
    contract_details.contract.acquire();

    // Check that contract value is correct
    assert_eq!(contract_details.contract.get_balance(true), scale);
}

// The value of a scale combinator with a negative scale value provided is correct
#[test]
fn scale_with_provided_negative_scale_value_has_correct_value() {
    // Create contract or scale -1 one
    let scale = -1;
    let mut contract_details = setup_contract(vec![5, 1, scale, 1]);

    ext_update(|e| e.sender(contract_details.holder));
    contract_details.contract.acquire();

    // Check that contract value is correct
    assert_eq!(contract_details.contract.get_balance(true), scale);
}

// The value of a scale combinator with an agreed-upon observable scale value is correct
#[test]
fn scale_with_concrete_obs_value_has_correct_value() {
    let arbiter: Address = "3D04E16e08E4c1c7fa8fC5A386237669341EaAcE".parse().unwrap();
    let arbiter_serialized: [i64; 4] = address_to_i64(arbiter);

    // Create contract or scale obs arbiter one
    let mut contract_details = setup_contract(vec![
        5, -1, arbiter_serialized[0], arbiter_serialized[1], arbiter_serialized[2], arbiter_serialized[3],
        1
    ]);

    // Propose obs_value_0 = 2 from the arbiter
    ext_update(|e| e.sender(arbiter));
    contract_details.contract.set_obs_value(0, 2);

    ext_update(|e| e.sender(contract_details.holder));
    contract_details.contract.acquire();

    // Check that contract value is 2
    assert_eq!(contract_details.contract.get_balance(true), 2);
}

// The value of a scale combinator with an agreed-upon negative observable scale value is correct
#[test]
fn scale_with_concrete_negative_obs_value_has_correct_value() {
    let arbiter: Address = "3D04E16e08E4c1c7fa8fC5A386237669341EaAcE".parse().unwrap();
    let arbiter_serialized: [i64; 4] = address_to_i64(arbiter);

    // Create contract or scale obs arbiter one
    let mut contract_details = setup_contract(vec![
        5, 0, arbiter_serialized[0], arbiter_serialized[1], arbiter_serialized[2], arbiter_serialized[3],
        1
    ]);

    // Propose obs_value_0 = -2 from the arbiter
    ext_update(|e| e.sender(arbiter));
    contract_details.contract.set_obs_value(0, -2);

    ext_update(|e| e.sender(contract_details.holder));
    contract_details.contract.acquire();

    // Check that contract value is -2
    assert_eq!(contract_details.contract.get_balance(true), -2);
}

// The value of a scale combinator with an agreed-upon observable scale value changes after a new agreed-upon proposal
#[test]
fn scale_with_concrete_obs_value_has_correct_value_after_second_agreement() {
    let arbiter: Address = "3D04E16e08E4c1c7fa8fC5A386237669341EaAcE".parse().unwrap();
    let arbiter_serialized: [i64; 4] = address_to_i64(arbiter);

    // Create contract or scale obs arbiter one
    let mut contract_details = setup_contract(vec![
        5, 0, arbiter_serialized[0], arbiter_serialized[1], arbiter_serialized[2], arbiter_serialized[3],
        1
    ]);


    // Propose obs_value_0 = 2 from the arbiter
    ext_update(|e| e.sender(arbiter));
    contract_details.contract.set_obs_value(0, 2);

    // Propose obs_value_0 = 3 from the arbiter
    contract_details.contract.set_obs_value(0, 3);

    // Check that contract value is now 3
    ext_update(|e| e.sender(contract_details.holder));
    contract_details.contract.acquire();

    assert_eq!(contract_details.contract.get_balance(true), 3);
}

// The value of a give contract is correct
#[test]
fn give_value_correct() {
    // Create contract give one
    let mut contract_details = setup_contract(vec![6, 1]);

    // Check that the contract value is -1
    ext_update(|e| e.sender(contract_details.holder));
    contract_details.contract.acquire();

    assert_eq!(contract_details.contract.get_balance(true), -1);
}

// The value of a then contract is the value of the first sub-combinator pre-expiry
#[test]
fn then_value_equals_first_sub_combinator_pre_expiry() {
    // Create contract then truncate 1 one zero
    let mut contract_details = setup_contract(vec![7, 4, 1, 1, 0]);

    // Check value equals 1
    ext_update(|e| e
        .timestamp(1)
        .sender(contract_details.holder)
    );
    contract_details.contract.acquire();

    assert_eq!(contract_details.contract.get_balance(true), 1);
}

// The value of a then contract is the value of the second sub-combinator post-expiry
#[test]
fn then_value_equals_second_sub_combinator_post_expiry() {
    // Create contract then truncate 1 one zero
    let mut contract_details = setup_contract(vec![7, 4, 1, 1, 0]);

    // Check value equals 0
    ext_update(|e| e
        .timestamp(2)
        .sender(contract_details.holder)
    );
    contract_details.contract.acquire();

    assert_eq!(contract_details.contract.get_balance(true), 0);
}

// The value of a get contract is the value of the sub-combinator post-expiry (if acquired pre-expiry)
#[test]
fn get_has_correct_value() {
    // Create contract get truncate 1 one
    let contract_details = setup_contract(vec![8, 4, 1, 1]);

    // Mock details
    ext_update(|e| e
        .timestamp(0)
        .sender(contract_details.holder)
    );

    // Check value is 0 before expiry
    let mut contract = contract_details.contract;
    contract.acquire();
    assert_eq!(contract.get_balance(true), 0);

    // Check value is 1 after expiry
    ext_update(|e| e.timestamp(1));
    contract.update();
    assert_eq!(contract.get_balance(true), 1);
}

// The value of an anytime contract is correct with no additional acquisition
#[test]
fn anytime_has_correct_value_no_additional_acquisition() {
    // Create contract anytime truncate 1 one
    let contract_details = setup_contract(vec![9, 4, 1, 1]);

    // Mock details
    ext_update(|e| e
        .timestamp(0)
        .sender(contract_details.holder)
    );

    // Check value is 0 before expiry
    let mut contract = contract_details.contract;
    contract.acquire();
    assert_eq!(contract.get_balance(true), 0);

    // Check value is 1 after expiry
    ext_update(|e| e.timestamp(1));
    contract.update();
    assert_eq!(contract.get_balance(true), 1);
}

// The value of an anytime contract is correct after additional acquisition
#[test]
fn anytime_has_correct_value_after_additional_acquisition() {
    // Create contract anytime truncate 5 one
    let contract_details = setup_contract(vec![9, 4, 5, 1]);

    // Mock details
    ext_update(|e| e
        .timestamp(0)
        .sender(contract_details.holder)
    );

    // Check value is 0 before acquisition
    let mut contract = contract_details.contract;
    contract.acquire();
    assert_eq!(contract.get_balance(true), 0);

    // Acquire anytime contract
    contract.acquire_anytime_sub_contract(0);

    // Check value is 1
    assert_eq!(contract.get_balance(true), 1);
}

// An expired contract should be concluded.
#[test]
fn expired_contract_concluded() {
    // Create contract truncate 0 one
    let mut contract_details = setup_contract(vec![4, 0, 1]);

    ext_update(|e| e.timestamp(1));
    assert!(contract_details.contract.get_concluded());
}

// A fully updated contract should be concluded.
#[test]
fn fully_updated_contract_concluded() {
    // Create contract one
    let contract_details = setup_contract(vec![1]);

    ext_update(|e| e
        .timestamp(0)
        .sender(contract_details.holder)
    );
    let mut contract = contract_details.contract;
    contract.acquire();

    assert!(contract.get_concluded());
}

// Acquiring an expired contract is not allowed
#[test]
#[should_panic(expected = "Cannot acquire an expired contract.")]
fn should_panic_if_acquiring_post_expiry() {
    // Initialise contract truncate 0 one
    let mut contract_details = setup_contract(vec![4, 0, 1]);

    // Attempt to acquire contract
    ext_update(|e| e
        .sender(contract_details.holder)
        .timestamp(1)
    );

    contract_details.contract.acquire();
}

// A concluded contract shouldn't be able to be updated.
#[test]
#[should_panic(expected = "Contract has concluded, nothing more to update.")]
fn should_panic_if_updating_concluded_contract() {
    // Create contract truncate 0 one
    let mut contract_details = setup_contract(vec![4, 0, 1]);

    ext_update(|e| e.timestamp(1));
    assert!(contract_details.contract.get_concluded());

    contract_details.contract.update();
}