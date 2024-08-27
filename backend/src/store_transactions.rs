use std::cell::RefCell;

use ic_cdk::update;

// A thread-local storage for transaction hashes
thread_local! {
    static TRANSACTION_HASHES: RefCell<Vec<String>> = RefCell::new(Vec::new());
}

// Function to store a transaction hash
#[update]
pub fn store_transaction_hash(tx_hash: String) {
    TRANSACTION_HASHES.with(|hashes| {
        hashes.borrow_mut().push(tx_hash);
    });
}

// Function to retrieve all stored transaction hashes
pub fn get_transaction_hashes() -> Vec<String> {
    TRANSACTION_HASHES.with(|hashes| {
        hashes.borrow().clone()
    })
}