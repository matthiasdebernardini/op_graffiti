#![deny(clippy::all)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![warn(clippy::cargo)]

//! This crate is meant to be used by webdevs to easily integrate into their projects
//! by running a microservice that allows them to run bdk
//! to make `op_return` transactions using bitcoin

mod error;
mod util;

use crate::util::{get_electrum_client, sync_electrum, NETWORK};
use axum::response::IntoResponse;
use axum::{extract::Path, routing::get, Json, Router};
use bdk_chain::{ChainPosition, ConfirmationTimeHeightAnchor};
use bdk_wallet::bitcoin::script::PushBytesBuf;
use bdk_wallet::bitcoin::{Amount, Txid};
use bdk_wallet::{floating_rate, KeychainKind, SignOptions, Wallet};
use better_panic::Settings;
use doc_comment::doc_comment;
use serde::{Serialize, Serializer};
use serde_json::json;
use tracing::info;

#[tracing::instrument]
async fn get_op_return() -> error::Result<impl IntoResponse> {
    let external_descriptor = "wpkh(tprv8ZgxMBicQKsPdy6LMhUtFHAgpocR8GC6QmwMSFpZs7h6Eziw3SpThFfczTDh5rW2krkqffa11UpX3XkeTTB2FvzZKWXqPY54Y6Rq4AQ5R8L/84'/1'/0'/0/*)";
    let internal_descriptor = "wpkh(tprv8ZgxMBicQKsPdy6LMhUtFHAgpocR8GC6QmwMSFpZs7h6Eziw3SpThFfczTDh5rW2krkqffa11UpX3XkeTTB2FvzZKWXqPY54Y6Rq4AQ5R8L/84'/1'/0'/1/*)";
    let mut wallet = Wallet::new(external_descriptor, internal_descriptor, NETWORK)?;
    sync_electrum(&mut wallet);

    let transactions = get_tx_details(&wallet).unwrap();

    let j = json!({ "transactions": transactions });

    Ok(Json(j))
}
#[derive(serde_derive::Serialize, Clone, Debug)]
pub struct TxDetail<'a> {
    pub received: Amount,
    pub sent: Amount,
    pub fee: Amount,
    pub fee_rate: f64,
    pub txid: Txid,
    #[serde(serialize_with = "serialize_chain_position")]
    pub chain_position: ChainPosition<&'a ConfirmationTimeHeightAnchor>,
}

pub(crate) fn serialize_chain_position<S>(
    chain_position: &ChainPosition<&ConfirmationTimeHeightAnchor>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match chain_position {
        ChainPosition::Confirmed(anchor) => {
            let confirmation_time = anchor.confirmation_time;
            let height = anchor.confirmation_height;
            let confirmed = json!({
                "type": "Confirmed",
                "anchor": {
                    "height": height,
                    "confirmation_time": confirmation_time,
                }
            });
            confirmed.serialize(serializer)
        }
        ChainPosition::Unconfirmed(ts) => {
            let unconfirmed = json!({
                "type": "Unconfirmed",
                "timestamp": ts,
            });
            unconfirmed.serialize(serializer)
        }
    }
}
pub fn get_tx_details(wallet: &Wallet) -> anyhow::Result<Vec<TxDetail>> {
    wallet
        .transactions()
        .map(|tx| {
            let txid = tx.tx_node.txid;
            let chain_position = tx.chain_position;
            let tx = tx.tx_node.tx.as_ref();
            let (sent, received) = wallet.sent_and_received(tx);
            let fee = wallet.calculate_fee(tx)?;
            let fee_rate = wallet.calculate_fee_rate(tx)?;
            let fee_rate = floating_rate!(fee_rate);

            let tx_detail = TxDetail {
                received,
                sent,
                fee,
                fee_rate,
                txid,
                chain_position,
            };

            Ok(tx_detail)
        })
        .collect()
}

#[tracing::instrument]
async fn write_op_return(Path(data): Path<String>) -> error::Result<impl IntoResponse> {
    info!("Received WRITE request with data: {}", data.clone());

    let external_descriptor = "wpkh(tprv8ZgxMBicQKsPdy6LMhUtFHAgpocR8GC6QmwMSFpZs7h6Eziw3SpThFfczTDh5rW2krkqffa11UpX3XkeTTB2FvzZKWXqPY54Y6Rq4AQ5R8L/84'/1'/0'/0/*)";
    let internal_descriptor = "wpkh(tprv8ZgxMBicQKsPdy6LMhUtFHAgpocR8GC6QmwMSFpZs7h6Eziw3SpThFfczTDh5rW2krkqffa11UpX3XkeTTB2FvzZKWXqPY54Y6Rq4AQ5R8L/84'/1'/0'/1/*)";
    let mut wallet = Wallet::new(external_descriptor, internal_descriptor, NETWORK)?;
    sync_electrum(&mut wallet);
    let address = wallet.next_unused_address(KeychainKind::External);

    info!("Deposit sats to this address: {}", address);
    let mut tx_builder = wallet.build_tx();

    let push_bytes = PushBytesBuf::try_from(data.into_bytes()).unwrap();

    tx_builder.add_data(&push_bytes);
    let mut psbt = tx_builder.finish()?;
    let finalized = wallet.sign(&mut psbt, SignOptions::default())?;
    assert!(finalized);

    let tx = psbt.extract_tx()?;
    let client = get_electrum_client();
    client.transaction_broadcast(&tx)?;

    let txid = tx.compute_txid().to_string();

    let j = json!({ "txid": txid });

    Ok(Json(j))
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    Settings::debug()
        .most_recent_first(false)
        .lineno_suffix(true)
        .install();

    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    let app = Router::new()
        .route("/get_op_return", get(get_op_return))
        .route("/write_op_return/:data", get(write_op_return));

    // let listener = tokio::net::TcpListener::bind("127.0.0.1:9000").await.unwrap();
    let addr = if cfg!(debug_assertions) {
        std::net::SocketAddr::from(([127, 0, 0, 1], 9000))
    } else {
        std::net::SocketAddr::from(([0, 0, 0, 0], 9000))
    };
    let listener = tokio::net::TcpListener::bind(addr).await?;
    info!("Server running on {:?}", listener);
    axum::serve(listener, app).await.unwrap();

    Ok(())
}

doc_comment!(concat!("fooo", "or not foo"), pub struct Foo {});
