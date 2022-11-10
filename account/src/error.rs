use thiserror::Error;

use crate::{Amount, Deposit, Withdraw};

// TODO: all errors should have the tx that caused them. and provide acc info
// fix this after you introduce the tracing app.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum Error {
    // Generic transaction application errors
    #[error("Account is locked. Transactions are not accepted")]
    LockedAccount,
    #[error("Primary transactions transaction id must be unique")]
    TransactionReplay,

    // Primary transactions application errors
    #[error("Available funds overflow")]
    DepositOverflow,
    #[error("Available funds overflow")]
    ResolveOverflow,
    #[error("Held funds overflow")]
    DisputeOverflow,
    #[error("Total funds overflow")]
    TotalOverflow,
    #[error("Not enough available funds to make a withdraw")]
    InsufficientFundsForWithdraw,
    #[error("Not enough available funds to make a dispute")]
    InsufficientFundsForDispute,

    // Dispute and dispute resolution errors. These include replay of secondary transactions.
    #[error("Target transaction id is not present in the set of all deposits")]
    InvalidDisputeTarget,
    #[error("Target transaction id is not present in the set of all deposits")]
    InvalidResolveTarget,
    #[error("Target transaction id is not present in the set of all deposits")]
    InvalidChargeBackTarget,

    #[error("Transaction is already disputed. Can't have more than one active dispute")]
    AlreadyDisputed,
    #[error("Charged back transactions can't be referenced by dispute or dispute resolution transactions")]
    AlreadyChargedBack,
    #[error("Dispute resolution transactions can be applied only to disputed transactions")]
    TargetNotDisputed,

    // TransactionRequest Errors
    #[error("Minimum allowed value for Deposits is {:#?}", Deposit::MIN)]
    InsufficientDepositAmount,
    #[error("Minimum allowed value for Withdraw is {:#?}", Withdraw::MIN)]
    InsufficientWithdrawAmount,

    #[error(
        "Amount has to be in the interval <{:#?}, {:#?}]",
        Amount::MIN,
        Amount::MAX
    )]
    AmountOutOfBounds,
}
