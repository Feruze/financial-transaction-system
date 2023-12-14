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
#[derive(candid::CandidType, Serialize, Deserialize, Default, Clone)]
struct AuditLogEntry {
    id: u64,
    action_type: ActionType,
    affected_account_id: u64,
    timestamp: u64,
    details: String,
}

#[derive(candid::CandidType, Serialize, Deserialize, Clone)]
enum ActionType {
    None,
    AccountCreation,
    AccountUpdate,
    TransactionExecution,
    RewardDistribution,
    // ... other action types
}

impl Default for ActionType {
    fn default() -> Self {
        ActionType::None
    }
}

impl Storable for AuditLogEntry {
    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
}

impl BoundedStorable for AuditLogEntry {
    const MAX_SIZE: u32 = 2048; // Adjust based on expected size
    const IS_FIXED_SIZE: bool = false;
}


#[derive(candid::CandidType, Clone, Serialize, Deserialize, Default)]
struct NotificationLogEntry {
    id: u64,
    account_id: u64,
    message: String,
    timestamp: u64,
}
impl Storable for NotificationLogEntry {
    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
}
impl BoundedStorable for NotificationLogEntry {
    const MAX_SIZE: u32 = 2048; // Adjust this based on the expected size of the log entry
    const IS_FIXED_SIZE: bool = false;
}

#[derive(candid::CandidType, Serialize, Deserialize, Default, Clone)]
struct Stake {
    account_id: u64,
    staked_amount: f64,
    staking_since: u64,
    staking_period: u64, // in seconds
}

// Implement storage-related traits for the Stake struct
impl Storable for Stake {
    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
}

impl BoundedStorable for Stake {
    const MAX_SIZE: u32 = 1024; // Adjust based on expected size
    const IS_FIXED_SIZE: bool = false;
}
// Thread-local storage for managing transactions
thread_local! {
    static TRANSACTIONS: RefCell<StableBTreeMap<u64, Transaction, Memory>> =
        RefCell::new(StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(2)))
    ));
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
    static NOTIFICATION_LOGS: RefCell<StableBTreeMap<u64, NotificationLogEntry, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(9))) // Assuming MemoryId::new(9) is for notification logs
        )
    );
    static AUDIT_LOGS: RefCell<StableBTreeMap<u64, AuditLogEntry, Memory>> = RefCell::new(
        StableBTreeMap::init(MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(9))))
    );
    static STAKES: RefCell<StableBTreeMap<u64, Stake, Memory>> = RefCell::new(
        StableBTreeMap::init(MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(10))))
    );
}
#[derive(candid::CandidType, Serialize, Deserialize, Default)]
struct NotificationLogEntryPayload {
    account_id: u64,
    message: String,
}

/// Represents the payload for transferring funds between two accounts.
#[derive(candid::CandidType, Serialize, Deserialize, Default)]
struct TransferPayload {
    sender_id: u64,
    receiver_id: u64,
    amount: f64,
}
// Payload for creating a new stake
#[derive(candid::CandidType, Serialize, Deserialize, Default)]
struct StakePayload {
    account_id: u64,
    amount: f64,
    staking_period: u64, // in seconds
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
const INTEREST_RATE: f64 = 0.01; // 1% interest rate, for example

#[ic_cdk::update]
fn apply_interest_to_all_accounts() -> Result<(), String> {
    // First, collect all account IDs
    let account_ids: Vec<u64> = ACCOUNTS.with(|accounts| {
        accounts.borrow().iter().map(|(id, _)| id).collect()
    });

    // Then, iterate over the IDs, updating accounts one by one
    for account_id in account_ids {
        ACCOUNTS.with(|accounts| {
            let mut accounts_map = accounts.borrow_mut();
            if let Some(mut account) = accounts_map.remove(&account_id) {
                // Calculate and apply interest
                let interest = account.balance * INTEREST_RATE;
                account.balance += interest;

                // Optionally, create a transaction record for the interest applied
                let interest_transaction = Transaction {
                    sender_id: 0, // System or bank ID
                    receiver_id: account_id,
                    amount: interest,
                    timestamp: time(),
                };

                // Insert the updated account and interest transaction
                accounts_map.insert(account_id, account);
                TRANSACTIONS.with(|transactions| {
                    transactions.borrow_mut().insert(interest_transaction.timestamp, interest_transaction);
                });
            }
        });
    }
    Ok(())
}
// Define a threshold for suspicious transactions
const SUSPICIOUS_AMOUNT_THRESHOLD: f64 = 10000.0; // Example threshold amount
const TIME_WINDOW: u64 = 86400; // 24 hours in seconds
const MAX_TRANSACTIONS_IN_WINDOW: usize = 10; // Example max transactions in 24 hours

#[ic_cdk::query]
fn check_for_suspicious_activity(account_id: u64) -> Vec<u64> {
    let mut suspicious_transactions = Vec::new();
    let now = time();

    // Filter transactions related to the account and within the time window
    let related_transactions = TRANSACTIONS.with(|transactions| {
        transactions.borrow()
            .iter()
            .filter(|(_, txn)| 
                (txn.sender_id == account_id || txn.receiver_id == account_id) &&
                now - txn.timestamp <= TIME_WINDOW)
            .map(|(id, _)| id)
            .collect::<Vec<u64>>()
    });

    // Check if the number of transactions exceeds the defined threshold
    if related_transactions.len() > MAX_TRANSACTIONS_IN_WINDOW {
        suspicious_transactions.extend_from_slice(&related_transactions);
    } else {
        // Check each transaction for amount threshold
        for txn_id in related_transactions {
            let txn = TRANSACTIONS.with(|txns| txns.borrow().get(&txn_id).unwrap().clone());
            if txn.amount > SUSPICIOUS_AMOUNT_THRESHOLD {
                suspicious_transactions.push(txn_id);
            }
        }
    }

    suspicious_transactions
}

#[ic_cdk::update]
fn reverse_transaction(transaction_id: u64) -> Result<Transaction, String> {
    let reverse_transaction_result = TRANSACTIONS.with(|txns| {
        if let Some(transaction) = txns.borrow().get(&transaction_id) {
            // Ensure the transaction amount is positive
            if transaction.amount <= 0.0 {
                return Err("Transaction amount must be positive for reversal".to_string());
            }

            // Process the reversal
            ACCOUNTS.with(|accs| {
                let mut accounts = accs.borrow_mut();

                // Temporarily remove sender and receiver accounts
                let mut sender_account = accounts.remove(&transaction.receiver_id).ok_or("Sender account not found".to_string())?;
                let mut receiver_account = accounts.remove(&transaction.sender_id).ok_or("Receiver account not found".to_string())?;

                // Check if receiver has enough balance
                if receiver_account.balance < transaction.amount {
                    return Err("Insufficient balance for reversal".to_string());
                }

                // Reverse the transaction
                sender_account.balance += transaction.amount;
                receiver_account.balance -= transaction.amount;

                // Reinsert the accounts
                accounts.insert(sender_account.id, sender_account);
                accounts.insert(receiver_account.id, receiver_account);

                // Create and insert the reverse transaction
                let reverse_transaction = Transaction {
                    sender_id: transaction.receiver_id,
                    receiver_id: transaction.sender_id,
                    amount: transaction.amount,
                    timestamp: ic_cdk::api::time(),
                };
                txns.borrow_mut().insert(reverse_transaction.timestamp, reverse_transaction.clone());

                Ok(reverse_transaction)
            })
        } else {
            Err("Transaction not found".to_string())
        }
    });

    reverse_transaction_result
}

#[ic_cdk::update]
fn create_log_entry(account_id: u64, message: String) -> Result<(), String> {
    let id = ID_COUNTER.with(|c| {
        let current_id = *c.borrow().get();
        let _ = c.borrow_mut().set(current_id + 1);
        current_id
    });

    let entry = NotificationLogEntry {
        id,
        account_id,
        message,
        timestamp: ic_cdk::api::time(),
    };

    NOTIFICATION_LOGS.with(|logs| logs.borrow_mut().insert(id, entry));
    Ok(())
}
#[ic_cdk::query]
fn get_logs() -> Vec<NotificationLogEntry> {
    NOTIFICATION_LOGS.with(|logs| logs.borrow().iter().map(|(_, entry)| entry.clone()).collect())
}

#[ic_cdk::update]
fn log_audit_entry(action_type: ActionType, affected_account_id: u64, details: String) -> Result<AuditLogEntry, String> {
    let id = ID_COUNTER.with(|counter| {
        let current_value = *counter.borrow().get();
        counter.borrow_mut().set(current_value + 1).unwrap();
        current_value
    });

    let entry = AuditLogEntry {
        id,
        action_type,
        affected_account_id,
        timestamp: ic_cdk::api::time(),
        details,
    };

    AUDIT_LOGS.with(|logs| {
        logs.borrow_mut().insert(id, entry.clone());
    });

    Ok(entry)
}

#[ic_cdk::query]
fn get_audit_logs() -> Vec<AuditLogEntry> {
    AUDIT_LOGS.with(|logs| {
        logs.borrow()
            .iter() // Or another method to iterate if `iter()` isn't available.
            .map(|(_key, value)| value.clone())
            .collect()
    })
}


// Function to create a new stake using StakePayload
#[ic_cdk::update]
fn create_stake(payload: StakePayload) -> Result<(), Error> {
    let current_time = time();
    let stake = Stake {
        account_id: payload.account_id,
        staked_amount: payload.amount,
        staking_since: current_time,
        staking_period: payload.staking_period,
    };

    // Ensure the account exists and has enough balance to stake
    let mut account = _get_account(&payload.account_id).ok_or(Error::NotFound {
        msg: "Account not found.".to_string(),
    })?;
    if account.balance < payload.amount {
        return Err(Error::InsufficientFunds {
            msg: "Insufficient funds to stake.".to_string(),
        });
    }
    account.balance -= payload.amount;

    // Insert the new stake and update the account balance
    STAKES.with(|stakes| {
        stakes.borrow_mut().insert(current_time, stake); // Use current_time as a unique key
    });
    do_insert_account(&account);

    Ok(())
}
#[ic_cdk::update]
fn calculate_and_distribute_rewards() -> Result<(), String> {
    let current_time = time();

    // Collect the updates to apply after iterating
    let mut updates = Vec::new();

    STAKES.with(|stakes| {
        let stakes_map = stakes.borrow();

        for (_key, stake) in stakes_map.iter() {
            if current_time >= (stake.staking_since + stake.staking_period) {
                // Calculate reward
                let reward = calculate_reward(stake.staked_amount, stake.staking_period);
                updates.push((stake.account_id, reward));
            }
        }
    });

    // Apply the collected updates
    for (account_id, reward) in updates {
        if let Some(mut account) = _get_account(&account_id) {
            account.balance += reward;
            do_insert_account(&account);

            // Optional: log the reward distribution in the audit log
            log_audit_entry(
                ActionType::RewardDistribution,
                account_id,
                format!("Distributed reward of {}", reward),
            ).unwrap();
        }
    }

    Ok(())
}

// Helper function to calculate rewards
fn calculate_reward(staked_amount: f64, staking_period: u64) -> f64 {
    // Define reward calculation logic here
    // This is a simple example using a fixed interest rate
    const REWARD_INTEREST_RATE: f64 = 0.05; // 5% interest rate
    staked_amount * (REWARD_INTEREST_RATE / 365.0) * (staking_period as f64 / 86400.0)
}

// Use existing helper functions _get_account, do_insert_account, and do_insert_transaction.

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