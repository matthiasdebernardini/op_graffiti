// External crate imports
use axum::extract::State;
use axum::{extract::Path, response::IntoResponse, Json};
use bdk_electrum::bdk_chain::bitcoin::script::PushBytesBuf;
use bdk_wallet::{KeychainKind, SignOptions, Wallet};
use serde_json::json;
use tracing::info;

// Local crate imports
use crate::error::{Graffiti, Report};
use crate::util::GrafittiState;
use crate::{
    error,
    util::{get_electrum_client, get_tx_details, sync_electrum, NETWORK},
    EXTERNAL_DESCRIPTOR, INTERNAL_DESCRIPTOR,
};

pub async fn get_op_return(State(gs): State<GrafittiState>) -> error::Result<impl IntoResponse> {
    info!("Received READ request for op return transactions");
    let mut wallet = Wallet::new(EXTERNAL_DESCRIPTOR, INTERNAL_DESCRIPTOR, NETWORK)?;
    let client = gs.blockchain.lock().await;

    sync_electrum(client, &mut wallet)
        .await
        .map_err(|e| Report::from(Graffiti::Anyhow(e)))?;

    let transactions = get_tx_details(&wallet).unwrap();

    let j = json!({ "transactions": transactions });

    Ok(Json(j))
}

pub async fn write_op_return(
    State(gs): State<GrafittiState>,
    Path(data): Path<String>,
) -> error::Result<impl IntoResponse> {
    info!("Received WRITE request with data: {}", &data);
    let client = gs.blockchain.lock().await;
    // let db = ss.bdk_pool.lock().await.clone();

    let mut wallet = Wallet::new(EXTERNAL_DESCRIPTOR, INTERNAL_DESCRIPTOR, NETWORK)?;
    sync_electrum(client, &mut wallet)
        .await
        .map_err(|e| Report::from(Graffiti::Anyhow(e)))?;

    let address = wallet.next_unused_address(KeychainKind::External);

    info!(
        "Deposit sats to this address in case the wallet is dry: {}",
        address
    );

    let mut tx_builder = wallet.build_tx();

    let push_bytes = PushBytesBuf::try_from(data.into_bytes())?;
    tx_builder.add_data(&push_bytes);

    let mut psbt = tx_builder.finish()?;
    let finalized = wallet.sign(&mut psbt, SignOptions::default())?;

    assert!(finalized);

    let tx = psbt.extract_tx()?;

    let client = get_electrum_client().map_err(|e| Report::from(Graffiti::Anyhow(e)))?;
    client.transaction_broadcast(&tx)?;

    let txid = tx.compute_txid();
    let j = json!({ "txid": txid });

    Ok(Json(j))
}
