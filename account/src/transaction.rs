use derive_more::From;

use crate::{account::AccountId, amount::Amount, Error};

#[cfg(feature = "serde")]
use serde::{self, Deserialize, Serialize};

// TODO: doc that the user is responsible for providing valid tx ids
#[derive(Clone, Debug, PartialEq, Eq, Hash, From)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct TransactionId(pub u32);

// Note:
// The problem does not require us to distinguish TransactionRequest from Transaction.
// If this were production code, it may be wise to make that distinction even if
// the benefits are not immediately obvious.
//cfg_attr(feature = "serde",

#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(
        into = "transaction_record::TransactionRecord",
        from = "transaction_record::TransactionRecord"
    )
)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Transaction {
    pub target_account_id: AccountId,
    pub kind: TransactionKind,
}

impl Transaction {
    // TODO: this allows to construct low invalid deposits
    pub fn deposit(
        target_account_id: AccountId,
        tx_id: TransactionId,
        amount: Amount,
    ) -> Result<Self, Error> {
        Ok(Transaction {
            target_account_id,
            kind: TransactionKind::Deposit(Deposit::new(tx_id, amount)?),
        })
    }

    pub fn withdraw(
        target_account_id: AccountId,
        tx_id: TransactionId,
        amount: Amount,
    ) -> Result<Self, Error> {
        Ok(Transaction {
            target_account_id,
            kind: TransactionKind::Withdraw(Withdraw::new(tx_id, amount)?),
        })
    }

    pub fn dispute(target_account_id: AccountId, target_tx_id: TransactionId) -> Self {
        Transaction {
            target_account_id,
            kind: TransactionKind::Dispute(Dispute { target_tx_id }),
        }
    }

    pub fn resolve(target_account_id: AccountId, target_tx_id: TransactionId) -> Self {
        Transaction {
            target_account_id,
            kind: TransactionKind::Resolve(Resolve { target_tx_id }),
        }
    }

    pub fn charge_back(target_account_id: AccountId, target_tx_id: TransactionId) -> Self {
        Transaction {
            target_account_id,
            kind: TransactionKind::ChargeBack(ChargeBack { target_tx_id }),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, From)]
pub enum TransactionKind {
    Deposit(Deposit),
    Withdraw(Withdraw),
    Dispute(Dispute),
    Resolve(Resolve),
    ChargeBack(ChargeBack),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Deposit {
    tx_id: TransactionId,
    amount: Amount,
}

impl Deposit {
    /// Minimum allowed value for Deposits
    pub const MIN: Amount = Amount::MIN;

    pub fn new(tx_id: TransactionId, amount: Amount) -> Result<Self, Error> {
        if amount <= Self::MIN {
            Err(Error::InsufficientDepositAmount)
        } else {
            Ok(Deposit { tx_id, amount })
        }
    }

    pub fn tx_id(&self) -> &TransactionId {
        &self.tx_id
    }

    pub fn to_tx_id(&self) -> TransactionId {
        self.tx_id.clone()
    }

    pub fn amount(&self) -> &Amount {
        &self.amount
    }

    pub fn to_amount(&self) -> Amount {
        self.amount.clone()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Withdraw {
    tx_id: TransactionId,
    amount: Amount,
}

impl Withdraw {
    /// Minimum allowed value for Withdrawals
    pub const MIN: Amount = Amount::MIN;

    pub fn new(tx_id: TransactionId, amount: Amount) -> Result<Self, Error> {
        if amount <= Self::MIN {
            Err(Error::InsufficientWithdrawAmount)
        } else {
            Ok(Withdraw { tx_id, amount })
        }
    }

    pub fn tx_id(&self) -> &TransactionId {
        &self.tx_id
    }

    pub fn to_tx_id(&self) -> TransactionId {
        self.tx_id.clone()
    }

    pub fn amount(&self) -> &Amount {
        &self.amount
    }

    pub fn to_amount(&self) -> Amount {
        self.amount.clone()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Dispute {
    pub target_tx_id: TransactionId,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Resolve {
    pub target_tx_id: TransactionId,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChargeBack {
    pub target_tx_id: TransactionId,
}

#[cfg(feature = "serde")]
mod transaction_record {
    use serde::{Deserialize, Serialize};

    use crate::{AccountId, Amount, Transaction, TransactionId, TransactionKind};

    #[derive(Serialize, Deserialize)]
    pub struct TransactionRecord<'a> {
        r#type: &'a str, // TODO: we check every byte record is utf8. do we need that? // try [u8]
        client: AccountId,
        tx: TransactionId,
        amount: Option<Amount>,
    }

    impl<'a> From<Transaction> for TransactionRecord<'a> {
        fn from(tx: Transaction) -> Self {
            let client = tx.target_account_id;
            let (r#type, tx, amount) = match tx.kind {
                TransactionKind::Deposit(deposit) => {
                    let amount = Some(deposit.amount);
                    let tx = deposit.tx_id;
                    let r#type = "deposit";
                    (r#type, tx, amount)
                }
                TransactionKind::Withdraw(withdraw) => {
                    let amount = Some(withdraw.amount);
                    let tx = withdraw.tx_id;
                    let r#type = "withdraw";
                    (r#type, tx, amount)
                }
                TransactionKind::Dispute(dispute) => {
                    let amount = None;
                    let tx = dispute.target_tx_id;
                    let r#type = "dispute";
                    (r#type, tx, amount)
                }
                TransactionKind::Resolve(resolve) => {
                    let amount = None;
                    let tx = resolve.target_tx_id;
                    let r#type = "resolve";
                    (r#type, tx, amount)
                }
                TransactionKind::ChargeBack(charge_back) => {
                    let amount = None;
                    let tx = charge_back.target_tx_id;
                    let r#type = "chargeback";
                    (r#type, tx, amount)
                }
            };

            TransactionRecord {
                r#type,
                client,
                tx,
                amount,
            }
        }
    }

    impl<'a> From<TransactionRecord<'a>> for Transaction {
        fn from(tx_record: TransactionRecord) -> Self {
            match tx_record.r#type {
                "deposit" => {
                    Transaction::deposit(tx_record.client, tx_record.tx, tx_record.amount.unwrap())
                        .unwrap()
                }
                "withdraw" => {
                    Transaction::withdraw(tx_record.client, tx_record.tx, tx_record.amount.unwrap())
                        .unwrap()
                }
                // TODO: assert that amount is none?
                "resolve" => Transaction::resolve(tx_record.client, tx_record.tx),
                "dispute" => Transaction::dispute(tx_record.client, tx_record.tx),
                "chargeback" => Transaction::charge_back(tx_record.client, tx_record.tx),
                _ => panic!(),
            }
        }
    }
}
