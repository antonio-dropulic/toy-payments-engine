use std::collections::HashMap;

use derive_more::From;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
#[cfg(feature = "tracing")]
use tracing;

use crate::{
    amount::Amount,
    error::Error,
    transaction::{
        ChargeBack, Deposit, Dispute, Resolve, Transaction, TransactionId, TransactionKind,
        Withdraw,
    },
};

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(PartialEq, Eq, Hash, Clone, Debug, From)]
pub struct AccountId(pub u16);

#[derive(Debug, Clone)]
enum DisputeState {
    /// Transaction that is currently not disputed. Transactions that
    /// had its disputed state resolved are considered not disputed.
    NotDisputed,
    /// Transaction that is currently disputed and awaiting resolution.
    Disputed,
    // Transaction that was ChargedBack. This transaction can no longer be disputed.
    ChargedBack,
}

#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IsLocked {
    #[cfg_attr(feature = "serde", serde(rename = "true"))]
    Locked,
    #[cfg_attr(feature = "serde", serde(rename = "false"))]
    Unlocked,
}
#[cfg_attr(feature = "serde", derive(Serialize))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AccountState {
    #[cfg_attr(feature = "serde", serde(rename = "client"))]
    pub id: AccountId,
    pub available: Amount,
    pub held: Amount,
    #[cfg_attr(feature = "serde", serde(rename = "locked"))]
    pub is_locked: IsLocked,
}

impl AccountState {
    // replace with a helper method in tests?
    // TODO: why this
    #[cfg(test)]
    pub(crate) fn new(
        id: impl Into<AccountId>,
        available: impl Into<Amount>,
        held: impl Into<Amount>,
        locked: IsLocked,
    ) -> Self {
        Self {
            id: id.into(),
            available: available.into(),
            held: held.into(),
            is_locked: locked,
        }
    }

    pub fn from_id(id: AccountId) -> Self {
        Self {
            id,
            available: Amount::from_u64(0),
            held: Amount::from_u64(0),
            is_locked: IsLocked::Unlocked,
        }
    }

    pub fn total(&self) -> Result<Amount, Error> {
        self.available
            .checked_add(&self.held)
            .ok_or(Error::TotalOverflow)
    }
}

#[derive(Debug, Clone)]
pub struct Account {
    /// account read model
    state: AccountState,
    /// extension to the account state so secondary transactions can be handled.
    deposits: HashMap<TransactionId, (Deposit, DisputeState)>,
    // this map scales linearly with the size of the input data
    // tx_id is any valid u32, that means we can have more than 4bil deposits
    // to keep track of. This will eat up memory.
    // i can maybe read lines from a file?
}

// methods implemented on the extended state (state + dispute map)
// all these methods need to be transactional! In other words,
// the state should never be left in an inconsistent state. If an
// error occurs no changes to the state can happen!
// TODO: splitting these methods into handle and apply where apply can't fail
// might be a better api.
impl Account {
    fn try_apply_deposit(&mut self, deposit: Deposit) -> Result<(), Error> {
        self.state.available = self
            .state
            .available
            .checked_add(deposit.amount())
            .ok_or(Error::DepositOverflow)?;

        // WARN: we don't check that the tx id is unique. That is assumed.
        // If the assumption does not hold execution of try apply tx is UD.
        let is_deposit_unique = self.deposits.insert(
            deposit.tx_id().clone(),
            (deposit, DisputeState::NotDisputed),
        );

        debug_assert!(matches!(is_deposit_unique, None));

        Ok(())
    }

    fn try_apply_withdraw(&mut self, withdraw: &Withdraw) -> Result<(), Error> {
        self.state.available = self
            .state
            .available
            .checked_sub(withdraw.amount())
            .ok_or(Error::InsufficientFundsForWithdraw)?;
        Ok(())
    }

    fn try_apply_dispute(&mut self, dispute: &Dispute) -> Result<(), Error> {
        // deposit should not be mutated
        let (deposit, dispute_state) = self
            .deposits
            .get_mut(&dispute.target_tx_id)
            .ok_or(Error::InvalidDisputeTarget)?;

        match dispute_state {
            DisputeState::NotDisputed => {
                let mut state = self.state.clone();

                // NOTE: making sure that the tx is transactional!
                let new_state = {
                    state.available = state
                        .available
                        .checked_sub(deposit.amount())
                        .ok_or(Error::InsufficientFundsForDispute)?;

                    state.held = state
                        .held
                        .checked_add(deposit.amount())
                        .ok_or(Error::DisputeOverflow)?;

                    state
                };

                self.state = new_state;

                *dispute_state = DisputeState::Disputed;

                Ok(())
            }
            DisputeState::Disputed => Err(Error::AlreadyDisputed),
            DisputeState::ChargedBack => Err(Error::AlreadyChargedBack),
        }
    }

    fn try_apply_resolve(&mut self, resolve: &Resolve) -> Result<(), Error> {
        // todo deposit should not be modified
        let (deposit, dispute_status) = self
            .deposits
            .get_mut(&resolve.target_tx_id)
            .ok_or(Error::InvalidResolveTarget)?;

        match dispute_status {
            DisputeState::Disputed => {
                self.state.available = self
                    .state
                    .available
                    .checked_add(deposit.amount())
                    .ok_or(Error::ResolveOverflow)?;

                // TODO: except note
                // The unwrap is fine here since there can be only one
                // active dispute on a transaction. That means that
                // this is the only resolution ever applied to that
                // for that dispute, effectively reversing it's effect.
                // held value has no other ways of changing.
                self.state.held = self.state.held.checked_sub(deposit.amount()).unwrap();

                *dispute_status = DisputeState::NotDisputed;
                Ok(())
            }
            DisputeState::NotDisputed => Err(Error::TargetNotDisputed),
            DisputeState::ChargedBack => Err(Error::AlreadyChargedBack),
        }
    }

    fn try_apply_charge_back(&mut self, charge_back: &ChargeBack) -> Result<(), Error> {
        // deposit should not be modified
        let (deposit, dispute_status) = self
            .deposits
            .get_mut(&charge_back.target_tx_id)
            .ok_or(Error::InvalidChargeBackTarget)?;

        match dispute_status {
            DisputeState::Disputed => {
                self.state.held = self
                    .state
                    .held
                    .checked_sub(deposit.amount())
                    // TODO: except note
                    // The unwrap is fine here since there can be only one
                    // active dispute on a transaction. That means that
                    // this is the only resolution ever applied to that
                    // for that dispute, effectively reversing it's effect.
                    // held value has no other ways of changing.
                    .unwrap();

                *dispute_status = DisputeState::ChargedBack;
                self.state.is_locked = IsLocked::Locked;
                Ok(())
            }
            DisputeState::NotDisputed => Err(Error::TargetNotDisputed),
            DisputeState::ChargedBack => Err(Error::AlreadyChargedBack),
        }
    }
}

impl Account {
    /// Creates an empty account with given id.
    pub fn from_id(id: AccountId) -> Self {
        Account {
            state: AccountState::from_id(id),
            deposits: HashMap::new(),
        }
    }

    #[cfg(test)]
    /// Returns an Account with the State defined with the given arguments.
    /// Account transactions are empty and thus inconsistent with the state.
    pub fn test_account(
        id: AccountId,
        available: Amount,
        held: Amount,
        is_locked: IsLocked,
    ) -> Self {
        Self {
            state: AccountState {
                id,
                available,
                held,
                is_locked,
            },
            deposits: HashMap::new(),
        }
    }

    pub fn state(&self) -> &AccountState {
        &self.state
    }

    pub fn into_state(self) -> AccountState {
        self.state
    }

    /// TODO: better name
    #[cfg_attr(feature = "tracing", tracing::instrument(err(Display)))]
    pub fn try_apply_transaction(&mut self, transaction_request: Transaction) -> Result<(), Error> {
        if self.state().is_locked == IsLocked::Locked {
            return Err(Error::LockedAccount);
        }

        // NOTE:
        // not checking tx_id uniqueness, as the assignment does not mandate it.

        // TODO: log state change!
        // TODO: refactor into handle / apply api to make the transactional nature
        // of these operations more obvious

        // handle tx
        match transaction_request.kind {
            TransactionKind::Deposit(ref deposit) => self.try_apply_deposit(deposit.clone())?,
            TransactionKind::Withdraw(ref withdraw) => self.try_apply_withdraw(withdraw)?,
            TransactionKind::Dispute(ref dispute) => self.try_apply_dispute(dispute)?,
            TransactionKind::Resolve(ref resolve) => self.try_apply_resolve(resolve)?,
            TransactionKind::ChargeBack(ref charge_back) => {
                self.try_apply_charge_back(charge_back)?;
            }
        };

        #[cfg(feature = "tracing")]
        tracing::info!(transaction = ?transaction_request, "Transaction applied");

        Ok(())
    }
}

#[cfg(feature = "serde")]
mod account_state_record {
    use super::{AccountState, Amount};
    use serde::Serialize;

    #[derive(Serialize)]
    struct AccountStateRecord {
        #[serde(flatten)]
        account_state: AccountState,
        total: Amount,
    }

    impl From<AccountState> for AccountStateRecord {
        fn from(account_state: AccountState) -> Self {
            AccountStateRecord {
                total: account_state.total().unwrap(),
                account_state,
            }
        }
    }
}
