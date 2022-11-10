mod account;
mod amount;
mod error;
mod transaction;

// TODO: cleanup imports
// TODO: what does the account symbol clash with
pub use crate::account::{Account, AccountId, AccountState, IsLocked};
pub use amount::Amount;
pub use error::Error;
pub use transaction::{
    ChargeBack, Deposit, Dispute, Resolve, Transaction, TransactionId, TransactionKind, Withdraw,
};

// TODO: move into test utils
// TODO: clean up, see what you need
#[cfg(test)]
mod test_helpers {
    // use super::{
    //     AccountId, Amount, ChargeBack, Deposit, Dispute, Resolve, Transaction, TransactionId,
    //     TransactionKind, Withdraw,
    // };

    /// Build an array of transactions targeted at a single account id.
    ///
    /// `Deposit` and `Withdraw` are provided amounts. Transaction id will be the index
    /// in the provided transaction array. `Dispute`, `Resolve`, `Chargeback` must be provided
    /// with a `target_tx_id`.
    #[macro_export]
    macro_rules! acc {
        ($id:expr, [$($tx:ident($arg:tt)),*]) => {
            // TODO: match branch for that
            // if no transactions are provided there is no mutation
            #[allow(unused_mut)]
            #[allow(unused_variables)]
            // last tx id assignment is unused.
            #[allow(unused_assignments)]
            {

                let mut account = Account::from_id(AccountId($id as u16));
                let mut tx_id = 0 as u32;

                $(
                    let transaction = Transaction {
                        target_account_id: AccountId($id as u16),
                        kind: $crate::match_tx!($tx{$arg}, tx_id)
                    };

                    account.try_apply_transaction(transaction).unwrap();

                    tx_id += 1;
                )*

                account
            }
            };
    }

    #[macro_export]
    macro_rules! match_tx {
        (Deposit {MAX}, $tx_id:expr) => {
            Deposit::new(TransactionId($tx_id as u32), Amount::MAX)
                .unwrap()
                .into()
        };
        (Deposit {$amount: expr}, $tx_id:expr) => {
            Deposit::new(
                TransactionId($tx_id as u32),
                Amount::from_u64($amount as u64),
            )
            .unwrap()
            .into()
        };
        (Withdraw {$amount: expr}, $tx_id:expr) => {
            Withdraw::new(
                TransactionId($tx_id as u32),
                Amount::from_u64($amount as u64),
            )
            .unwrap()
            .into()
        };
        (Dispute {$target_tx: expr}, $tx_id:expr) => {
            Dispute {
                target_tx_id: TransactionId($target_tx as u32),
            }
            .into()
        };
        (Resolve {$target_tx: expr}, $tx_id:expr) => {
            Resolve {
                target_tx_id: TransactionId($target_tx as u32),
            }
            .into()
        };
        (ChargeBack {$target_tx: expr}, $tx_id:expr) => {
            ChargeBack {
                target_tx_id: TransactionId($target_tx as u32),
            }
            .into()
        };
    }
}

#[cfg(test)]
mod tests {
    use test_case::test_case;

    use super::*;
    use crate::transaction::{Deposit, Withdraw};

    fn test_success(mut account: Account, tx: Transaction, expected_state: AccountState) {
        account.try_apply_transaction(tx).expect("testing success");

        // it is important these assertions always hold
        // this way we check that in case of failure no changes to the account
        // are made
        assert_eq!(account.state(), &expected_state);
    }

    fn test_failure(mut account: Account, tx: Transaction) -> Error {
        let expected_state = account.state().clone();

        let err = account.try_apply_transaction(tx.clone()).unwrap_err();

        // it is important these assertions always hold
        // this way we check that in case of failure no changes to the account
        // are made
        assert_eq!(account.state(), &expected_state);

        err
    }

    #[test]
    fn create_empty_account() {
        let target_account_id = AccountId(0);

        let new_account = Account::from_id(target_account_id.clone());

        let state = new_account.state();

        assert_eq!(state.id, target_account_id);
        assert_eq!(state.available, Amount::from_u64(0));
        assert_eq!(state.held, Amount::from_u64(0));
        assert_eq!(state.is_locked, IsLocked::Unlocked);
    }

    // #[test_case(
    //     AccountId(0),
    //     Transaction::deposit(AccountId(0), TransactionId(0), Amount::MIN),
    //     AccountState::new(AccountId(0), Amount::MIN, Amount::from_u64(0), IsLocked::Unlocked)
    //     => Err(Error::InsufficientDepositAmount)
    // ; "Deposit min amount")]

    #[test_case(
        AccountId(0),
        Transaction::deposit(AccountId(0), TransactionId(0), Amount::from_u64(10)).unwrap(),
        AccountState::new(AccountId(0), Amount::from_u64(10), Amount::from_u64(0), IsLocked::Unlocked)
    ; "Deposit 10")]
    #[test_case(
        AccountId(0),
        Transaction::deposit(AccountId(0), TransactionId(0), Amount::from_u64(u64::MAX)).unwrap(),
        AccountState::new(AccountId(0), Amount::from_u64(u64::MAX), Amount::from_u64(0), IsLocked::Unlocked)
    ; "Deposit u64::max")]
    #[test_case(
        AccountId(0),
        Transaction::deposit(AccountId(0), TransactionId(0), Amount::MAX).unwrap(),
        AccountState::new(AccountId(0), Amount::MAX, Amount::from_u64(0), IsLocked::Unlocked)
    ; "Deposit Amount::MAX")]
    fn deposit_into_empty_account(
        // opening an account
        account_id: AccountId,
        tx: Transaction,
        expected_state: AccountState,
    ) {
        test_success(Account::from_id(account_id), tx, expected_state)
    }

    #[test_case(
        Account::test_account(AccountId(0), Amount::from_u64(100), Amount::from_u64(0), IsLocked::Unlocked),
        Transaction::deposit(AccountId(0), TransactionId(0), Amount::from_u64(10)).unwrap(),
        AccountState::new(AccountId(0), Amount::from_u64(110), Amount::from_u64(0), IsLocked::Unlocked)
    ; "Deposit with no held amount")]
    #[test_case(
        Account::test_account(AccountId(0), Amount::from_u64(100), Amount::MAX, IsLocked::Unlocked),
        Transaction::deposit(AccountId(0), TransactionId(0), Amount::from_u64(10)).unwrap(),
        AccountState::new(AccountId(0), Amount::from_u64(110), Amount::MAX, IsLocked::Unlocked)
    ; "Deposit with some held")]
    #[test_case(
        Account::test_account(AccountId(0), Amount::from_u64(100), Amount::from_u64(0), IsLocked::Unlocked),
        Transaction::deposit(AccountId(0), TransactionId(0), Amount::MAX.checked_sub(&Amount::from_u64(100)).unwrap()).unwrap(),
        AccountState::new(AccountId(0), Amount::MAX, Amount::from_u64(0), IsLocked::Unlocked)
    ; "Deposit max")]
    fn deposit_success(account: Account, tx: Transaction, expected_state: AccountState) {
        test_success(account, tx, expected_state)
    }

    #[test_case(
        Account::test_account(AccountId(0), Amount::from_u64(1), Amount::from_u64(0), IsLocked::Unlocked),
        Transaction::deposit(AccountId(0), TransactionId(0), Amount::MAX).unwrap()
        => Error::DepositOverflow
    ; "Deposit with overflow")]
    fn deposit_failure(account: Account, tx: Transaction) -> Error {
        test_failure(account, tx)
    }

    #[test_case(
        Account::test_account(AccountId(0), Amount::from_u64(150), Amount::from_u64(0), IsLocked::Unlocked),
        Transaction::withdraw(AccountId(0), TransactionId(0), Amount::from_u64(149)).unwrap(),
        AccountState::new(AccountId(0), Amount::from_u64(1), Amount::from_u64(0), IsLocked::Unlocked)
    ; "Withdraw from account with sufficient funds")]
    #[test_case(
        Account::test_account(AccountId(0), Amount::from_u64(150), Amount::from_u64(10), IsLocked::Unlocked),
        Transaction::withdraw(AccountId(0), TransactionId(0), Amount::from_u64(149)).unwrap(),
        AccountState::new(AccountId(0), Amount::from_u64(1), Amount::from_u64(10), IsLocked::Unlocked)
    ; "Withdraw from account with sufficient and some funds held")]
    #[test_case(
        Account::test_account(AccountId(0), Amount::from_u64(200), Amount::from_u64(0), IsLocked::Unlocked),
        Transaction::withdraw(AccountId(0), TransactionId(0), Amount::from_u64(200)).unwrap(),
        AccountState::new(AccountId(0), Amount::from_u64(0), Amount::from_u64(0), IsLocked::Unlocked)
    ; "Withdraw everything")]
    // #[test_case(
    //     Account::test_account(AccountId(0), Amount::from_u64(200), Amount::from_u64(0), IsLocked::Unlocked),
    //     Transaction::withdraw(AccountId(0), TransactionId(0), Amount::from_u64(0)),
    //     AccountState::new(AccountId(0), Amount::from_u64(200), Amount::from_u64(0), IsLocked::Unlocked)
    //     => Err(Error::InsufficientWithdrawAmount)
    // ; "Withdraw min amount")]
    fn withdraw_success(account: Account, tx: Transaction, expected_state: AccountState) {
        test_success(account, tx, expected_state)
    }

    #[test_case(
        Account::from_id(AccountId(0)),
        Transaction::withdraw(AccountId(0), TransactionId(0), Amount::from_u64(10)).unwrap()
        => Error::InsufficientFundsForWithdraw
    ; "Withdraw from empty account")]
    #[test_case(
        Account::test_account(AccountId(0), Amount::from_u64(100), Amount::from_u64(0), IsLocked::Unlocked),
        Transaction::withdraw(AccountId(0), TransactionId(0), Amount::from_u64(101)).unwrap()
        => Error::InsufficientFundsForWithdraw
    ; "Withdraw from with insufficient funds")]
    fn withdraw_failure(account: Account, tx: Transaction) -> Error {
        test_failure(account, tx)
    }

    #[test_case(
        acc!(0, [Deposit(10), Withdraw(10), Deposit(5)]),
        Transaction::dispute(AccountId(0), TransactionId(3))
        => Error::InvalidDisputeTarget
    ; "Dispute non existing transaction")]
    #[test_case(
        acc!(0, [Deposit(10), Withdraw(10), Deposit(5)]),
        Transaction::dispute(AccountId(0), TransactionId(1))
        => Error::InvalidDisputeTarget
    ; "Dispute withdraw")]
    #[test_case(
        acc!(0, [Deposit(10), Withdraw(10), Deposit(5), Dispute(2)]),
        Transaction::dispute(AccountId(0), TransactionId(3))
        => Error::InvalidDisputeTarget
    ; "Dispute dispute")]
    #[test_case(
        acc!(0, [Deposit(10), Withdraw(10), Deposit(5), Dispute(2), Resolve(2)]),
        Transaction::dispute(AccountId(0), TransactionId(4))
        => Error::InvalidDisputeTarget
    ; "Dispute Resolve")]
    #[test_case(
        acc!(0, [Deposit(10), Withdraw(10), Deposit(5), Dispute(2), ChargeBack(2)]),
        Transaction::dispute(AccountId(0), TransactionId(1))
        => Error::LockedAccount
    ; "Dispute ChargeBack")]
    #[test_case(
        acc!(0, [Deposit(10), Withdraw(10), Deposit(5)]),
        Transaction::dispute(AccountId(0), TransactionId(0))
        => Error::InsufficientFundsForDispute
    ; "Dispute with insufficient funds")]
    #[test_case(
        acc!(0, [Deposit(1), Dispute(0), Deposit(MAX)]),
        Transaction::dispute(AccountId(0), TransactionId(2))
        => Error::DisputeOverflow
    ; "Dispute with overflow")]

    fn dispute_failure(account: Account, tx: Transaction) -> Error {
        test_failure(account, tx)
    }

    #[test_case(
        acc!(0, [Deposit(15)]),
        Transaction::dispute(AccountId(0), TransactionId(0)),
        AccountState::new(AccountId(0), Amount::from_u64(0), Amount::from_u64(15), IsLocked::Unlocked)
    ; "Dispute a single deposit")]
    #[test_case(
        acc!(0, [Deposit(15), Withdraw(10), Deposit(10)]),
        Transaction::dispute(AccountId(0), TransactionId(0)),
        AccountState::new(AccountId(0), Amount::from_u64(0), Amount::from_u64(15), IsLocked::Unlocked)
    ; "Dispute after withdraws have been made")]
    #[test_case(
        acc!(0, [Deposit(15), Dispute(0), Resolve(0)]),
        Transaction::dispute(AccountId(0), TransactionId(0)),
        AccountState::new(AccountId(0), Amount::from_u64(0), Amount::from_u64(15), IsLocked::Unlocked)
    ; "Dispute same deposit multiple times")]
    fn dispute_success(account: Account, tx: Transaction, expected_state: AccountState) {
        test_success(account, tx, expected_state)
    }

    #[test_case(
        acc!(0, [Deposit(10)]),
        Transaction::resolve(AccountId(0), TransactionId(1))
        => Error::InvalidResolveTarget
    ; "Resolve non existing transaction")]
    #[test_case(
        acc!(0, []),
        Transaction::resolve(AccountId(0), TransactionId(0))
        => Error::InvalidResolveTarget
    ; "Resolve non existing transaction simple")]
    #[test_case(
        acc!(0, [Deposit(10)]),
        Transaction::resolve(AccountId(0), TransactionId(0))
        => Error::TargetNotDisputed
    ; "Resolve non disputed Deposit")]
    #[test_case(
        acc!(0, [Deposit(20), Deposit(10), Withdraw(10), Dispute(0)]),
        Transaction::resolve(AccountId(0), TransactionId(2))
        => Error::InvalidResolveTarget
    ; "Resolve withdraw")]
    #[test_case(
        acc!(0, [Deposit(10), Dispute(0)]),
        Transaction::resolve(AccountId(0), TransactionId(1))
        => Error::InvalidResolveTarget
    ; "Resolve dispute")]
    #[test_case(
        acc!(0, [Deposit(10), Dispute(0), Resolve(0)]),
        Transaction::resolve(AccountId(0), TransactionId(2))
        => Error::InvalidResolveTarget
    ; "Resolve resolve")]
    #[test_case(
        acc!(0, [Deposit(10), Dispute(0), ChargeBack(0)]),
        Transaction::resolve(AccountId(0), TransactionId(2))
        => Error::LockedAccount
    ; "Resolve charge back")]
    #[test_case(
        acc!(0, [Deposit(MAX), Dispute(0), Deposit(1)]),
        Transaction::resolve(AccountId(0), TransactionId(0))
        => Error::ResolveOverflow
    ; "Resolve with overflow")]
    fn resolve_failure(account: Account, tx: Transaction) -> Error {
        test_failure(account, tx)
    }

    #[test_case(
        acc!(0, [Deposit(15), Dispute(0)]),
        Transaction::resolve(AccountId(0), TransactionId(0)),
        AccountState::new(AccountId(0), Amount::from_u64(15), Amount::from_u64(0), IsLocked::Unlocked)
    ; "Resolve once")]
    #[test_case(
        acc!(0, [Deposit(15), Dispute(0), Resolve(0), Dispute(0)]),
        Transaction::resolve(AccountId(0), TransactionId(0)),
        AccountState::new(AccountId(0), Amount::from_u64(15), Amount::from_u64(0), IsLocked::Unlocked)
    ; "Resolve many times")]
    fn resolve_success(account: Account, tx: Transaction, expected_state: AccountState) {
        test_success(account, tx, expected_state)
    }

    #[test_case(
        acc!(0, [Deposit(10)]),
        Transaction::charge_back(AccountId(0), TransactionId(1))
        => Error::InvalidChargeBackTarget
    ; "Charge back non existing transaction")]
    #[test_case(
        acc!(0, []),
        Transaction::charge_back(AccountId(0), TransactionId(0))
        => Error::InvalidChargeBackTarget
    ; "Charge back non existing transaction simple")]
    #[test_case(
        acc!(0, [Deposit(10)]),
        Transaction::resolve(AccountId(0), TransactionId(0))
        => Error::TargetNotDisputed
    ; "Charge back non disputed Deposit")]
    #[test_case(
        acc!(0, [Deposit(20), Deposit(10), Withdraw(10), Dispute(0)]),
        Transaction::charge_back(AccountId(0), TransactionId(2))
        => Error::InvalidChargeBackTarget
    ; "Charge back withdraw")]
    #[test_case(
        acc!(0, [Deposit(10), Dispute(0)]),
        Transaction::charge_back(AccountId(0), TransactionId(1))
        => Error::InvalidChargeBackTarget
    ; "Charge back dispute")]
    #[test_case(
        acc!(0, [Deposit(10), Dispute(0), Resolve(0)]),
        Transaction::charge_back(AccountId(0), TransactionId(2))
        => Error::InvalidChargeBackTarget
    ; "Charge back resolve")]
    #[test_case(
        acc!(0, [Deposit(10), Dispute(0), ChargeBack(0)]),
        Transaction::charge_back(AccountId(0), TransactionId(2))
        => Error::LockedAccount
    ; "Charge back charge back")]

    fn charge_back_failure(account: Account, tx: Transaction) -> Error {
        test_failure(account, tx)
    }

    #[test_case(
        acc!(0, [Deposit(15), Dispute(0)]),
        Transaction::charge_back(AccountId(0), TransactionId(0)),
        AccountState::new(AccountId(0), Amount::from_u64(0), Amount::from_u64(0), IsLocked::Locked)
    ; "Charge back")]
    fn charge_back_success(account: Account, tx: Transaction, expected_state: AccountState) {
        test_success(account, tx, expected_state)
    }
}
