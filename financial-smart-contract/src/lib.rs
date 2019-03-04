#![no_std]
#![allow(non_snake_case)]
#![feature(proc_macro_hygiene)]

extern crate pwasm_ethereum;
extern crate pwasm_abi;

use pwasm_abi::eth::EndpointInterface;

mod financial_sc;
mod combinators;

// Executed when the contract is called
#[no_mangle]
pub fn call() {
    // Dispatch contract call to contract endpoint with given input, return result
    let contract = financial_sc::FinancialScContract::new();
    let mut endpoint = financial_sc::FinancialScEndpoint::new(contract);
    pwasm_ethereum::ret(&endpoint.dispatch(&pwasm_ethereum::input()));
}

// Executed when the contract is deployed
#[no_mangle]
pub fn deploy() {
    // Dispatch contract constructor call with given input
    let contract = financial_sc::FinancialScContract::new();
    let mut endpoint = financial_sc::FinancialScEndpoint::new(contract);
    endpoint.dispatch_ctor(&pwasm_ethereum::input());
}
