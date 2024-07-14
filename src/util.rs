// Third-party crates
use axum::routing::get;
use axum::Router;
use better_panic::Settings;
use doc_comment::doc_comment;
use serde::{Serialize, Serializer};
use serde_json::json;
use tokio::net::TcpListener;
use tracing::info;

// BDK (Bitcoin Development Kit) related imports
use bdk_electrum::bdk_chain::{ChainPosition, ConfirmationTimeHeightAnchor};
use bdk_electrum::electrum_client::Client;
use bdk_electrum::BdkElectrumClient;
use bdk_wallet::bitcoin::script::Instruction;
use bdk_wallet::bitcoin::Network::{Bitcoin, Regtest, Signet, Testnet};
use bdk_wallet::bitcoin::{Amount, Network, Txid};
use bdk_wallet::{floating_rate, Wallet};
// Local imports
use crate::routes::{get_op_return, write_op_return};

pub const NETWORK: Network = {
    if cfg!(feature = "bitcoin") {
        Bitcoin
    } else if cfg!(feature = "regtest") {
        Regtest
    } else if cfg!(feature = "testnet") {
        Testnet
    } else {
        Signet
    }
};
const STOP_GAP: usize = 50;
const BATCH_SIZE: usize = 5;

#[tracing::instrument]
pub async fn sync_electrum(wallet: &mut Wallet) -> anyhow::Result<()> {
    info!("syncing electrum");
    let client = get_electrum_client()?;

    // Populate the electrum client's transaction cache so it doesn't redownload transaction we
    // already have.
    client.populate_tx_cache(&wallet);

    let request = wallet.start_full_scan();

    // todo might be slow
    let mut update = client
        .full_scan(request, STOP_GAP, BATCH_SIZE, false)?
        // .unwrap()
        .with_confirmation_time_height_anchor(&client)?;
    // .unwrap();

    let now = std::time::UNIX_EPOCH.elapsed().unwrap().as_secs();
    let _ = update.graph_update.update_last_seen_unconfirmed(now);

    wallet.apply_update(update)?;
    Ok(())
}

pub fn get_electrum_client() -> anyhow::Result<BdkElectrumClient<Client>> {
    let electrum_url = match NETWORK {
        Bitcoin => "ssl://electrum.blockstream.info:50002",
        // "ssl://mempool.space:50002",
        _ => "ssl://mempool.space:60602",
    };

    let client = Client::new(electrum_url)?;
    let client = BdkElectrumClient::new(client);
    Ok(client)
}

doc_comment!(
    r#"
    # Transaction Detail

    `TxDetail` represents the detailed information of a Bitcoin transaction.

    This struct encapsulates various aspects of a transaction, including the amounts
    involved, the transaction ID, and its position in the blockchain.

    ## Fields

    * `received`: The total amount of Bitcoin received in this transaction.
    * `sent`: The total amount of Bitcoin sent in this transaction.
    * `fee`: The transaction fee paid.
    * `fee_rate`: The fee rate in satoshis per virtual byte (sat/vB).
    * `txid`: The unique identifier of this transaction.
    * `chain_position`: The position of this transaction in the blockchain,
      including confirmation status and block height if confirmed.

    ## Serialization

    This struct derives `Serialize` for easy conversion to various data formats.
    Note that `chain_position` uses a custom serialization method.

    ## Usage

    `TxDetail` is typically used when querying transaction history or when detailed
    information about a specific transaction is required.

    ```rust
    let tx_detail = TxDetail {
        received: Amount::from_sat(100000),
        sent: Amount::from_sat(95000),
        fee: Amount::from_sat(5000),
        fee_rate: 10.5,
        txid: Txid::from_str("1234...").unwrap(),
        chain_position: ChainPosition::Confirmed(ConfirmedAt { height: 700000, time: 1234567890 }),
    };
    ```
    "#,
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
);

fn serialize_chain_position<S>(
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

/// # Errors
///
/// Will return errors if there is data missing
/// fetches details and formats the response
pub fn get_tx_details(wallet: &Wallet) -> anyhow::Result<Vec<TxDetail>> {
    wallet
        .transactions()
        .filter(|tx| {
            tx.tx_node.tx.output.iter().any(|output| {
                output
                    .script_pubkey
                    .instructions()
                    .next()
                    .map_or(false, |instruction| {
                        matches!(
                            instruction,
                            Ok(Instruction::Op(
                                bdk_wallet::bitcoin::blockdata::opcodes::all::OP_RETURN
                            ))
                        )
                    })
            })
        })
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

pub fn setup_better_panic() {
    Settings::debug()
        .most_recent_first(false)
        .lineno_suffix(true)
        .install();
}

pub async fn setup_server() -> anyhow::Result<(Router, TcpListener)> {
    let app = setup_router();

    let listener = setup_listener().await?;
    Ok((app, listener))
}

async fn setup_listener() -> anyhow::Result<TcpListener> {
    let addr = if cfg!(debug_assertions) {
        // Bind to localhost (127.0.0.1) for debug builds
        std::net::SocketAddr::from(([127, 0, 0, 1], 9000))
    } else {
        // Bind to all interfaces (0.0.0.0) for release builds
        std::net::SocketAddr::from(([0, 0, 0, 0], 9000))
    };
    let listener = tokio::net::TcpListener::bind(addr).await?;
    Ok(listener)
}

fn setup_router() -> Router {
    Router::new()
        .route("/get_op_return", get(get_op_return))
        .route("/write_op_return/:data", get(write_op_return))
}

pub fn setup_tracer() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();
}
