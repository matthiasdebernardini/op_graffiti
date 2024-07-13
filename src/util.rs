use bdk_electrum::electrum_client::Client;
use bdk_electrum::{electrum_client, BdkElectrumClient};
use bdk_wallet::bitcoin::Network;
use bdk_wallet::bitcoin::Network::{Bitcoin, Regtest, Signet, Testnet};
use bdk_wallet::{KeychainKind, Wallet};
use std::collections::HashSet;
use std::io::Write;

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
pub(crate) const STOP_GAP: usize = 50;
pub(crate) const BATCH_SIZE: usize = 5;

pub fn sync_electrum(wallet: &mut Wallet) {
    let client = get_electrum_client();

    // Populate the electrum client's transaction cache so it doesn't redownload transaction we
    // already have.
    client.populate_tx_cache(&wallet);

    let request = wallet
        .start_full_scan()
        .inspect_spks_for_all_keychains({
            let mut once = HashSet::<KeychainKind>::new();
            move |k, spk_i, _| {
                if once.insert(k) {
                    print!("\nScanning keychain [{:?}]", k)
                } else {
                    print!(" {:<3}", spk_i)
                }
            }
        })
        .inspect_spks_for_all_keychains(|_, _, _| std::io::stdout().flush().expect("must flush"));

    // todo might be slow
    let mut update = client
        .full_scan(request, STOP_GAP, BATCH_SIZE, false)
        .unwrap()
        .with_confirmation_time_height_anchor(&client)
        .unwrap();

    let now = std::time::UNIX_EPOCH.elapsed().unwrap().as_secs();
    let _ = update.graph_update.update_last_seen_unconfirmed(now);

    wallet.apply_update(update).unwrap();
}

pub fn get_electrum_client() -> BdkElectrumClient<Client> {
    let electrum_url = match NETWORK {
        // todo point to mempool
        Bitcoin => "ssl://electrum.blockstream.info:50002",
        // "ssl://mempool.space:50002",
        _ => "ssl://mempool.space:60602",
    };

    BdkElectrumClient::new(electrum_client::Client::new(electrum_url).unwrap())
}
