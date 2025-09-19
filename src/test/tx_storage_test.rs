use std::fs;
use std::sync::Arc;
use crate::chain::storage::db;
use crate::chain::storage::tx_storage::TxStorage;
use crate::chain::tx::Tx;
use crate::test::commons::{config, wallet, wallet_with_balance};

#[test]
fn test_tx_storage_save_find_by_hash() -> Result<(), std::io::Error> {
    let config = config();
    let db = db::open(&config).unwrap();
    let tx_storage = TxStorage::new(Arc::clone(&db));
    let from = wallet(&config);
    let to = wallet(&config);
    let tx = Tx::new(&from, to.address(), String::from("10"), 1)?;
    tx_storage.save(&tx)?;
    if let Some(found) = tx_storage.find_by_hash(tx.hash_str())? {
        assert_eq!(found, tx);
    } else {
        assert!(false);
    }
    if let Ok(found) = tx_storage.find_wallet_txs(tx.from()) {
        assert_eq!(found.len(), 1);
    } else {
        assert!(false);
    }
    if let None = tx_storage.find_by_hash(String::from(""))? {
        assert!(true);
    } else {
        assert!(false);
    }
    let tx = Tx::new(&from, to.address(), String::from("1"), 1)?;
    tx_storage.save(&tx)?;
    if let Ok(found) = tx_storage.find_wallet_txs(tx.from()) {
        assert_eq!(found.len(), 2);
    } else {
        assert!(false);
    }
    let txs = tx_storage.find_wallet_txs(String::from("wallet"))?;
    assert_eq!(txs.len(), 0);

    let pending = tx_storage.find_pending()?;
    assert_eq!(pending.len(), 2);

    tx_storage.update_pending(&pending, 1)?;
    let txs = tx_storage.find_pending()?;
    assert_eq!(txs.len(), 0);

    fs::remove_dir_all(config.storage_path())?;
    Ok(())
}

#[test]
fn update_pending() {
    let config = config();
    let wallet = wallet_with_balance(&config).unwrap();
    let db = db::open(&config).unwrap();
    let tx = Tx::new(&wallet, wallet.address(), String::from("10"), 1).unwrap();
    let tx_storage = TxStorage::new(Arc::clone(&db));
    tx_storage.save(&tx).unwrap();
    let txs = tx_storage.find_pending().unwrap();
    assert_eq!(txs.len(), 1);
    tx_storage.update_pending(&txs, 1).unwrap();
    let txs = tx_storage.find_pending().unwrap();
    assert_eq!(txs.len(), 0);
    fs::remove_dir_all(config.storage_path()).unwrap();
}
