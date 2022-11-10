use std::collections::HashMap;

use account::{Account, AccountId, AccountState, Transaction};

use async_std::{
    channel::{self, Receiver, Sender},
    prelude::*,
    stream,
    task::{self, JoinHandle},
};
use futures::{stream::FuturesUnordered, StreamExt};
#[cfg(feature = "tracing")]
use tracing;

/// This error should never happen. This must be satisfied by inspection.
const CLOSED_CHANNEL_ERROR: &str = "Existing accounts must have open channels";

#[derive(Debug)]
struct AccountHandler {
    sender: Sender<Transaction>,
    handler: JoinHandle<Account>,
}

pub async fn transaction_broker_sync(
    mut transaction_requests: impl Stream<Item = Transaction> + Unpin,
) -> impl Stream<Item = AccountState> {
    let mut accounts: HashMap<AccountId, Account> = HashMap::new();

    while let Some(tx_request) = transaction_requests.next().await {
        if let Some(account) = accounts.get_mut(&tx_request.target_account_id) {
            let _res = account.try_apply_transaction(tx_request);
        } else {
            let account_id = tx_request.target_account_id.clone();
            let mut new_account = Account::from_id(account_id.clone());
            let _res = new_account.try_apply_transaction(tx_request);
            accounts.insert(account_id, new_account);
        }
    }

    stream::from_iter(accounts.into_values()).map(Account::into_state)
}

#[cfg_attr(feature = "tracing", tracing::instrument(skip_all))]
pub async fn transaction_broker(
    mut transaction_requests: impl Stream<Item = Transaction> + Unpin,
) -> impl Stream<Item = AccountState> {
    let mut account_handlers: HashMap<AccountId, AccountHandler> = HashMap::new();

    // Note: sequentially handling the input stream. Assignment defined transaction
    // order to be the csv item order.
    while let Some(tx_request) = transaction_requests.next().await {
        if let Some(account_handler) = account_handlers.get(&tx_request.target_account_id) {
            send_tx(account_handler, tx_request).await;
        } else {
            let account_handler = start_account_handler(tx_request.clone());
            send_tx(&account_handler, tx_request.clone()).await;
            account_handlers.insert(tx_request.target_account_id, account_handler);
        }
    }

    join_account_handlers(account_handlers)
}

fn start_account_handler(tx_request: Transaction) -> AccountHandler {
    // Note: using unbounded channels for convenience.
    // In practice account we could control how much throughput is
    // allowed per account.
    let (sender, receiver) = channel::unbounded();
    let handler = task::spawn(transaction_listener(tx_request.target_account_id, receiver));

    AccountHandler { sender, handler }
}

/// # Errors
/// Trying to send to a closed channel is silently ignored.
/// Debug builds will panic.
#[cfg_attr(feature = "tracing", tracing::instrument(skip_all))]
async fn send_tx(account: &AccountHandler, tx_request: Transaction) {
    let _res = account.sender.send(tx_request).await;

    #[cfg(feature = "tracing")]
    if _res.is_err() {
        tracing::error!(error = %CLOSED_CHANNEL_ERROR);
    }

    debug_assert!(matches!(_res, Ok(())), "{}", CLOSED_CHANNEL_ERROR);
}

/// # Errors
/// All handlers are expected to have open channels. Already closed channels
/// are ignored. Debug builds will panic.
#[cfg_attr(feature = "tracing", tracing::instrument(skip_all))]
fn join_account_handlers(
    account_handlers: HashMap<AccountId, AccountHandler>,
) -> impl Stream<Item = AccountState> {
    account_handlers
        .into_iter()
        .map(|(_, account_handler)| {
            let _closed = account_handler.sender.close();

            #[cfg(feature = "tracing")]
            if _closed {
                tracing::error!(error = %CLOSED_CHANNEL_ERROR);
            }

            debug_assert!(_closed, "{}", CLOSED_CHANNEL_ERROR);

            account_handler.handler
        })
        .collect::<FuturesUnordered<_>>()
        .map(Account::into_state)
}

#[cfg_attr(feature = "tracing", tracing::instrument(skip(receiver), ret))]
async fn transaction_listener(account_id: AccountId, receiver: Receiver<Transaction>) -> Account {
    let mut account_aggregate = Account::from_id(account_id);
    #[cfg(feature = "tracing")]
    tracing::info!("Opening the account");

    while let Ok(tx_request) = receiver.recv().await {
        #[cfg(feature = "tracing")]
        tracing::info!(request = ?tx_request, "Transaction request received");
        let _res = account_aggregate.try_apply_transaction(tx_request);
    }

    #[cfg(feature = "tracing")]
    tracing::info!("Transaction listener closing");

    account_aggregate
}
