// Import necessary libraries and modules
#[macro_use]
extern crate serde;
use candid::{Decode, Encode};
use ic_cdk::api::time;
use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory};
use ic_stable_structures::{BoundedStorable, Cell, DefaultMemoryImpl, StableBTreeMap, Storable};
use std::{borrow::Cow, cell::RefCell};

// Define type aliases for clarity
type Memory = VirtualMemory<DefaultMemoryImpl>;
type IdCell = Cell<u64, Memory>;

/// Represents a user account with an ID, holder name, balance, and creation timestamp.
#[derive(candid::CandidType, Clone, Serialize, Deserialize, Default)]
struct Account {
    id: u64,
    holder_name: String,
    balance: f64,
    created_at: u64,
}

// Implement storage-related traits for the Account struct
impl Storable for Account {
    // Convert the Account struct to a byte representation
    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    // Convert a byte representation to an Account struct
    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
}

impl BoundedStorable for Account {
    const MAX_SIZE: u32 = 1024;
    const IS_FIXED_SIZE: bool = false;
}

// Thread-local storage for managing accounts
thread_local! {
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> = RefCell::new(
        MemoryManager::init(DefaultMemoryImpl::default())
    );

    static ID_COUNTER: RefCell<IdCell> = RefCell::new(
        IdCell::init(MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(0))), 0)
            .expect("Cannot create an ID counter")
    );

    static ACCOUNTS: RefCell<StableBTreeMap<u64, Account, Memory>> =
        RefCell::new(StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(1)))
    ));
}

/// Represents a financial transaction between two accounts.
#[derive(candid::CandidType, Serialize, Deserialize, Default, Clone)]
struct Transaction {
    sender_id: u64,
    receiver_id: u64,
    amount: f64,
    timestamp: u64,
}

// Implement storage-related traits for the Transaction struct
impl Storable for Transaction {
    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
}

impl BoundedStorable for Transaction {
    const MAX_SIZE: u32 = 1024;
    const IS_FIXED_SIZE: bool = false;
}

// Thread-local storage for managing transactions
thread_local! {
    static TRANSACTIONS: RefCell<StableBTreeMap<u64, Transaction, Memory>> =
        RefCell::new(StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(2)))
    ));
}

/// Represents the payload for transferring funds between two accounts.
#[derive(candid::CandidType, Serialize, Deserialize, Default)]
struct TransferPayload {
    sender_id: u64,
    receiver_id: u64,
    amount: f64,
}

/// Updates the global state to create a new account with the provided details.
#[ic_cdk::update]
fn create_account(holder_name: String, initial_balance: f64) -> Option<Account> {
    // Generate a new unique account ID
    let id = ID_COUNTER
        .with(|counter| {
            let current_value = *counter.borrow().get();
            counter.borrow_mut().set(current_value + 1)
        })
        .expect("Cannot increment ID counter");

    // Create a new account with the provided details
    let account = Account {
        id,
        holder_name,
        balance: initial_balance,
        created_at: time(),
    };

    // Insert the new account into the global storage
    do_insert_account(&account);

    // Return the created account
    Some(account)
}

// Helper function to insert an account into the global storage
fn do_insert_account(account: &Account) {
    ACCOUNTS.with(|service| service.borrow_mut().insert(account.id, account.clone()));
}

/// Retrieves the account with the specified ID from the global state.
#[ic_cdk::query]
fn get_account(id: u64) -> Result<Account, Error> {
    // Attempt to retrieve the account with the specified ID
    match _get_account(&id) {
        Some(account) => Ok(account),
        None => Err(Error::NotFound {
            msg: format!("Account with id={} not found.", id),
        }),
    }
}

// Helper function to retrieve an account from the global storage
fn _get_account(id: &u64) -> Option<Account> {
    ACCOUNTS.with(|s| s.borrow().get(id))
}

/// Updates the global state to transfer funds between two accounts.
#[ic_cdk::update]
fn transfer_funds(payload: TransferPayload) -> Result<Transaction, Error> {
    // Retrieve sender and receiver accounts from the global state
    let sender_account_option: Option<Account> = _get_account(&payload.sender_id);
    let receiver_account_option: Option<Account> = _get_account(&payload.receiver_id);

    // Match on both account options
    match (sender_account_option, receiver_account_option) {
        (Some(mut sender_account), Some(mut receiver_account)) => {
            // Check if the sender has sufficient funds
            if sender_account.balance >= payload.amount {
                // Update sender and receiver balances
                sender_account.balance -= payload.amount;
                receiver_account.balance += payload.amount;

                // Create a new transaction record
                let transaction = Transaction {
                    sender_id: payload.sender_id,
                    receiver_id: payload.receiver_id,
                    amount: payload.amount,
                    timestamp: time(),
                };

                // Insert the new transaction and update both accounts
                do_insert_transaction(&transaction);
                do_insert_account(&sender_account);
                do_insert_account(&receiver_account);

                // Return the created transaction
                Ok(transaction)
            } else {
                // Sender has insufficient funds
                Err(Error::InsufficientFunds {
                    msg: "Insufficient funds in the sender's account.".to_string(),
                })
            }
        }
        _ => {
            // Either sender or receiver account not found
            Err(Error::NotFound {
                msg: "Sender or receiver account not found.".to_string(),
            })
        }
    }
}

// Helper function to insert a transaction into the global storage
fn do_insert_transaction(transaction: &Transaction) {
    TRANSACTIONS.with(|service| service.borrow_mut().insert(transaction.timestamp, transaction.clone()));
}

/// Retrieves all transactions from the global state.
#[ic_cdk::query]
fn get_all_transactions() -> Result<Vec<Transaction>, Error> {
    // Retrieve all transactions and convert them to a Vec
    let transactions_map: Vec<(u64, Transaction)> =
        TRANSACTIONS.with(|service| service.borrow().iter().collect());
    let transactions: Vec<Transaction> = transactions_map.into_iter().map(|(_, txn)| txn).collect();

    // Check if any transactions were found
    if !transactions.is_empty() {
        // Return the list of transactions
        Ok(transactions)
    } else {
        // No transactions found
        Err(Error::NotFound {
            msg: "No transactions found.".to_string(),
        })
    }
}

/// Retrieves the account with the specified ID from the global state.
#[ic_cdk::query]
fn get_sender_account(sender_id: u64) -> Result<Account, Error> {
    // Attempt to retrieve the sender account with the specified ID
    match _get_account(&sender_id) {
        Some(account) => Ok(account),
        None => Err(Error::NotFound {
            msg: format!("Sender account with id={} not found.", sender_id),
        }),
    }
}

/// Retrieves the account with the specified ID from the global state.
#[ic_cdk::query]
fn get_receiver_account(receiver_id: u64) -> Result<Account, Error> {
    // Attempt to retrieve the receiver account with the specified ID
    match _get_account(&receiver_id) {
        Some(account) => Ok(account),
        None => Err(Error::NotFound {
            msg: format!("Receiver account with id={} not found.", receiver_id),
        }),
    }
}

/// Retrieves the balance of the account with the specified ID from the global state.
#[ic_cdk::query]
fn get_account_balance(id: u64) -> Result<f64, Error> {
    // Attempt to retrieve the account with the specified ID and return its balance
    match _get_account(&id) {
        Some(account) => Ok(account.balance),
        None => Err(Error::NotFound {
            msg: format!("Account with id={} not found.", id),
        }),
    }
}

/// Retrieves the creation timestamp of the account with the specified ID from the global state.
#[ic_cdk::query]
fn get_account_created_at(id: u64) -> Result<u64, Error> {
    // Attempt to retrieve the account with the specified ID and return its creation timestamp
    match _get_account(&id) {
        Some(account) => Ok(account.created_at),
        None => Err(Error::NotFound {
            msg: format!("Account with id={} not found.", id),
        }),
    }
}

/// Retrieves all accounts from the global state.
#[ic_cdk::query]
fn get_all_accounts() -> Vec<Account> {
    // Retrieve all accounts and convert them to a Vec
    let accounts_map: Vec<(u64, Account)> =
        ACCOUNTS.with(|service| service.borrow().iter().collect());
    let accounts: Vec<Account> = accounts_map.into_iter().map(|(_, acc)| acc).collect();
    accounts
}

/// Updates the global state to update the holder name of the account with the specified ID.
#[ic_cdk::update]
fn update_account_holder_name(id: u64, new_holder_name: String) -> Result<(), Error> {
    // Attempt to retrieve the account with the specified ID
    match _get_account(&id) {
        Some(mut account) => {
            // Update the holder name and insert the modified account back into the global state
            account.holder_name = new_holder_name;
            do_insert_account(&account);
            Ok(())
        }
        None => {
            // Account not found
            Err(Error::NotFound {
                msg: format!("Account with id={} not found.", id),
            })
        }
    }
}

/// Updates the global state to delete the account with the specified ID.
#[ic_cdk::update]
fn delete_account(id: u64) -> Result<(), Error> {
    // Check if the account with the specified ID exists
    if let Some(_account) = _get_account(&id) {
        // Remove the account from the global state
        ACCOUNTS.with(|service| service.borrow_mut().remove(&id));
        Ok(())
    } else {
        // Account not found
        Err(Error::NotFound {
            msg: format!("Account with id={} not found.", id),
        })
    }
}

/// Represents possible errors that can occur during account operations.
#[derive(candid::CandidType, Deserialize, Serialize)]
enum Error {
    /// Indicates that the requested resource was not found.
    NotFound { msg: String },
    /// Indicates that there are insufficient funds for a particular operation.
    InsufficientFunds { msg: String },
}

// Export Candid interface for the defined functions and types
ic_cdk::export_candid!();
