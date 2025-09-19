use crate::chain::block::Block;
use crate::chain::tx::Tx;
use crate::chain::wallet::Wallet;

#[test]
fn test_genesis_block_creation() -> Result<(), std::io::Error> {
    let wallet = Wallet::new();
    let mut txs = Vec::new();
    let tx = Tx::new(&wallet, wallet.address(), String::from("1"), 1)?;
    txs.push(tx);

    let block1 = Block::genesis(txs.clone());
    assert_eq!(block1.idx, 0);

    let block2 = Block::genesis(txs);
    assert_eq!(block2.idx, 0);

    assert_eq!(block1.hash(), block2.hash());
    assert_eq!(block1.hash_str(), block2.hash_str());
    Ok(())
}

#[test]
fn test_new_block_creation() -> Result<(), std::io::Error> {
    let wallet = Wallet::new();
    let mut txs = Vec::new();
    let tx = Tx::new(&wallet, wallet.address(), String::from("1"), 1)?;
    txs.push(tx);

    let genesis = Block::genesis(txs.clone());
    let block = Block::new(&wallet, 1, genesis.hash_str(), txs)?;
    Ok(())
}
