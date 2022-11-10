use account::{AccountState, Transaction};
use async_std::stream::{self, StreamExt};

use futures::Stream;
#[cfg(feature = "tracing")]
use tracing;

// TODO: use PATH instead of file name?
// TODO: error handling
// TODO: belongs to another file
#[cfg_attr(feature = "tracing", tracing::instrument)]
pub async fn process_csv_txs(input_file_name: &str, output_file_name: &str) {
    let records = txs_from_csv(input_file_name)
        .await
        .expect("Must be able to open file");

    let states = crate::transaction_broker(records).await;

    accounts_into_csv(output_file_name, states)
        .await
        .expect("Must be able to open file");
}

pub async fn process_csv_txs_sync(input_file_name: &str, output_file_name: &str) {
    let records = txs_from_csv(input_file_name)
        .await
        .expect("Must be able to open file");

    let states = crate::transaction_broker_sync(records).await;

    accounts_into_csv(output_file_name, states)
        .await
        .expect("Must be able to open file");
}

/// Read a csv file and deserialize it using serde.
/// Reader is buffered. Fields are assigned based on headers.
/// If a record can't be deserialized it is ignored.
///
/// # Errors
/// Error::FailedToOpenFile
#[cfg_attr(feature = "tracing", tracing::instrument)]
pub async fn txs_from_csv(input_file_name: &str) -> Result<impl Stream<Item = Transaction>, ()> {
    let file = async_std::fs::File::open(input_file_name)
        .await
        .map_err(|_e| ())?;
    let csv_reader = csv_async::AsyncDeserializer::from_reader(file);

    // might be better to let the caller decide what to do with errors
    Ok(csv_reader.into_deserialize().flat_map(|result| {
        #[cfg(feature = "tracing")]
        if result.is_err() {
            tracing::error!(err = ?result, "Failed to deserialize record");
        }

        stream::from_iter(result)
    }))
}

/// Write the account states to a CSV file.
/// Serialization is done using serde.
/// Serialization errors are ignored.
#[cfg_attr(feature = "tracing", tracing::instrument(skip(account_states)))]
pub async fn accounts_into_csv(
    output_file_name: &str,
    mut account_states: impl Stream<Item = AccountState> + Unpin,
) -> Result<(), ()> {
    let dst_file = async_std::fs::File::create(output_file_name)
        .await
        .map_err(|_e| ())?;
    let mut wtr = csv_async::AsyncSerializer::from_writer(dst_file);

    // TODO: https://docs.rs/tabwriter/1.2.1/tabwriter/
    // feature gate pretty print
    while let Some(state) = account_states.next().await {
        let _res = wtr.serialize(state).await;
        #[cfg(feature = "tracing")]
        if _res.is_err() {
            tracing::error!(err = ?_res, "Failed to serialize record");
        }
    }

    Ok(())
}
