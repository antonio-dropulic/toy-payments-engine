#![allow(clippy::all)]
#![allow(warnings)]

use core::num;
use std::{fs::File, path::Path};

use account::{AccountId, Amount, Transaction, TransactionId};

fn generate_test_csv(file_name: &str, tx_iter: impl Iterator<Item = Transaction>) {
    if Path::new(file_name).exists() {
        // don't recreate test csv's
        return ();
    }

    let test_file = File::create(file_name).unwrap();

    // TODO: tabwritter for pretty output
    // https://docs.rs/tabwriter/1.2.1/tabwriter/
    let mut wtr = csv::WriterBuilder::new().from_writer(test_file);

    for tx in tx_iter {
        wtr.serialize(tx).unwrap();
    }

    wtr.flush().unwrap();
}

/// Minimal transaction record data for use in test case generation.
/// Transaction id and Account id are provided during generation.
#[derive(Clone)]
pub enum TransactionRequestCompressed {
    Deposit(u64),    // amount
    Withdraw(u64),   // amount
    Dispute(u32),    // index of the target account in the pattern
    Resolve(u32),    // index of the target account in the pattern
    ChargeBack(u32), // index of the target account in the pattern
}

/// Create a vec of transactions by repeating the pattern in cycles.
/// Transaction index determines the transaction id.
/// # Panics
/// Trying to create more than u32::MAX transactions will result in a panic
pub fn pattern_iter(
    account_id: AccountId,
    pattern: Vec<TransactionRequestCompressed>,
    num_of_cycles: u32,
) -> impl Iterator<Item = Transaction> {
    let pattern_len = pattern.len();

    let into_tx = move |(index, tx_compressed): (usize, _)| match tx_compressed {
        TransactionRequestCompressed::Deposit(amount) => Transaction::deposit(
            account_id.clone(),
            TransactionId(index as u32),
            Amount::from_u64(amount),
        )
        .unwrap(),
        TransactionRequestCompressed::Withdraw(amount) => Transaction::withdraw(
            account_id.clone(),
            TransactionId(index as u32),
            Amount::from_u64(amount),
        )
        .unwrap(),
        TransactionRequestCompressed::Dispute(pattern_index) => {
            let current_cycle = index / pattern_len;
            let pattern_start_index = current_cycle * pattern_len;

            Transaction::dispute(
                account_id.clone(),
                TransactionId(pattern_index + pattern_start_index as u32),
            )
        }
        TransactionRequestCompressed::Resolve(pattern_index) => {
            let current_cycle = index / pattern_len;
            let pattern_start_index = current_cycle * pattern_len;

            Transaction::resolve(
                account_id.clone(),
                TransactionId(pattern_index + pattern_start_index as u32),
            )
        }
        TransactionRequestCompressed::ChargeBack(pattern_index) => {
            let current_cycle = index / pattern_len;
            let pattern_start_index = current_cycle * pattern_len;

            Transaction::charge_back(
                account_id.clone(),
                TransactionId(pattern_index + pattern_start_index as u32),
            )
        }
    };

    pattern
        .into_iter()
        .cycle()
        .enumerate()
        .take(num_of_cycles as usize * pattern_len)
        .map(into_tx)
}

// basic transaction patterns
// each pattern has the same num of tx so the bench can be comparable
// num of tx set to avoid too much outlier benchmarks

pub fn generate_deposits(file_name: &str) {
    generate_test_csv(
        file_name,
        pattern_iter(
            AccountId(0),
            vec![TransactionRequestCompressed::Deposit(1)],
            180,
        ),
    )
}

pub fn generate_deposit_withdraw(file_name: &str) {
    generate_test_csv(
        file_name,
        pattern_iter(
            AccountId(0),
            vec![
                TransactionRequestCompressed::Deposit(2),
                TransactionRequestCompressed::Withdraw(1),
            ],
            90,
        ),
    )
}

pub fn generate_deposit_dispute_resolve(file_name: &str) {
    generate_test_csv(
        file_name,
        pattern_iter(
            AccountId(0),
            vec![
                TransactionRequestCompressed::Deposit(2),
                TransactionRequestCompressed::Dispute(0),
                TransactionRequestCompressed::Resolve(0),
            ],
            60,
        ),
    )
}

pub fn generate_deposits_many_acc(file_name: &str) {
    generate_test_csv(
        file_name,
        TransposeFlatten::new(100, vec![TransactionRequestCompressed::Deposit(1)], 180),
    )
}

pub fn generate_deposits_withdraw_many_acc(file_name: &str) {
    generate_test_csv(
        file_name,
        TransposeFlatten::new(
            100,
            vec![
                TransactionRequestCompressed::Deposit(2),
                TransactionRequestCompressed::Withdraw(1),
            ],
            90,
        ),
    )
}

pub fn generate_deposits_dispute_resolve_many_acc(file_name: &str) {
    generate_test_csv(
        file_name,
        TransposeFlatten::new(
            100,
            vec![
                TransactionRequestCompressed::Deposit(2),
                TransactionRequestCompressed::Dispute(0),
                TransactionRequestCompressed::Resolve(0),
            ],
            60,
        ),
    )
}

/// flatten account tx in a round robin fashion
struct TransposeFlatten<I> {
    accounts: Vec<I>,
    index: usize,
    len: u16,
}

impl TransposeFlatten<Box<dyn Iterator<Item = Transaction>>> {
    fn new(
        num_of_acc: u16,
        pattern: Vec<TransactionRequestCompressed>,
        num_of_cycles: u32,
    ) -> Self {
        let mut new = Self {
            accounts: Vec::with_capacity(num_of_acc as usize),
            index: 0,
            len: num_of_acc,
        };

        for account_id in 0..num_of_acc {
            // TODO: can remove this box if I add exact types to pattern iter return value
            let acc_iter: Box<dyn Iterator<Item = Transaction>> = Box::new(pattern_iter(
                AccountId(account_id),
                pattern.clone(),
                num_of_cycles,
            ));

            new.accounts.push(acc_iter);
        }

        new
    }
}

impl<I> Iterator for TransposeFlatten<I>
where
    I: Iterator<Item = Transaction>,
{
    type Item = Transaction;

    fn next(&mut self) -> Option<Self::Item> {
        let next = self.accounts[self.index].next();
        self.index = (self.index + 1) % self.len as usize;
        next
    }
}
