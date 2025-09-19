use crate::chain::blockchain::Blockchain;
use crate::chain::tx::Tx;
use crate::test::commons::{config, wallet_with_balance};
use bigdecimal::BigDecimal;
use std::fs;

#[test]
fn test_blockchain() {
    let config = config();
    let wallet = wallet_with_balance(&config).unwrap();
    let blockchain = Blockchain::new(wallet.clone(), &config).unwrap();

    let balance = blockchain.balance(wallet.address()).unwrap();
    assert_eq!(balance, BigDecimal::from(500000));

    let nonce = blockchain.nonce(wallet.address()).unwrap();
    assert_eq!(nonce, 1);

    let tx = Tx::new(&wallet, String::from("to"), String::from("100.99"), 2).unwrap();
    blockchain.add_tx(&tx).unwrap();

    let tx = Tx::new(&wallet, String::from("to"), String::from("100.99"), 2).unwrap();
    assert!(blockchain.add_tx(&tx).is_err());

    let genesis = blockchain.find_block_by_idx(0).unwrap().unwrap();
    assert!(genesis.txs.is_some());
    assert_eq!(genesis.txs.unwrap().len(), 2);

    fs::remove_file(config.genesis_path()).unwrap();
    fs::remove_dir_all(config.storage_path()).unwrap();
}
