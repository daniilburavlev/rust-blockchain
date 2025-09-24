use std::fs;
use std::sync::Arc;
use tempfile::tempdir;
use wallet::wallet::Wallet;
use crate::blockchain::block::Block;
use crate::blockchain::storage::block_storage::BlockStorage;
use crate::blockchain::storage::db;
use chain::tx::Tx;
use crate::test::commons::config;

#[test]
fn test_block_save() {
    let temp_dir = tempdir().unwrap();
    let wallet = Wallet::new();
    let tx = Tx::new(&wallet, wallet.address(), String::from("0.001"), 1).unwrap();
    let txs = vec![tx];
    let block = Block::genesis(txs);

    let config = config(temp_dir.path());
    let db = db::open(&config).unwrap();
    let block_storage = BlockStorage::new(Arc::clone(&db));
    block_storage.save(&block).unwrap();

    if let Some(found) = block_storage.find_by_idx(0).unwrap() {
        assert_eq!(found.hash_str(), block.hash_str());
    } else {
        assert!(false, "Not found");
    }

    if let Some(found) = block_storage.find_by_hash(block.hash_str()).unwrap() {
        assert_eq!(found.hash_str(), block.hash_str());
    } else {
        assert!(false, "Not found");
    }

    if let Ok(latest) = block_storage.find_latest() {
        assert_eq!(block.hash_str(), latest.hash_str());
    } else {
        assert!(false, "Not found");
    }

    fs::remove_dir_all(config.storage_path()).unwrap();
}
