use serde::{Deserialize, Serialize};
use sha2::Digest;
use wallet::wallet::Wallet;
use chain::tx::Tx;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Block {
    pub idx: u64,
    pub timestamp: u64,
    pub validator: String,
    pub parent_hash: String,
    pub merkle_root: String,
    pub txs: Option<Vec<Tx>>,
    pub signature: String,
}

impl Block {
    pub fn new(
        wallet: &Wallet,
        idx: u64,
        parent_hash: String,
        txs: Vec<Tx>,
    ) -> Result<Block, std::io::Error> {
        let tx_hashes: Vec<[u8; 32]> = txs.clone().iter().map(|tx| tx.hash()).collect();
        let merkle_tree =
            rs_merkle::MerkleTree::<rs_merkle::algorithms::Sha256>::from_leaves(&tx_hashes);
        let merkle_root = if let Some(merkle_root) = merkle_tree.root() {
            merkle_root
        } else {
            [0u8; 32]
        };
        let mut block = Block {
            idx,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            validator: wallet.address(),
            parent_hash,
            merkle_root: hex::encode(merkle_root),
            txs: Some(txs),
            signature: String::from(""),
        };
        block.signature = wallet.sign(&block.hash())?;
        Ok(block)
    }

    pub fn genesis(txs: Vec<Tx>) -> Self {
        let tx_hashes: Vec<[u8; 32]> = txs.clone().iter().map(|tx| tx.hash()).collect();
        let merkle_tree =
            rs_merkle::MerkleTree::<rs_merkle::algorithms::Sha256>::from_leaves(&tx_hashes);
        let merkle_root = merkle_tree.root().unwrap();
        let validator = [0u8; 33];
        let parent_hash = [0u8; 32];
        Block {
            idx: 0,
            timestamp: txs.get(0).unwrap().timestamp,
            validator: hex::encode(validator),
            parent_hash: hex::encode(parent_hash),
            merkle_root: hex::encode(merkle_root),
            txs: Some(txs),
            signature: String::from("GENESIS"),
        }
    }

    pub fn txs(&self) -> Option<Vec<Tx>> {
        if let Some(txs) = self.txs.clone() {
            Some(txs)
        } else {
            None
        }
    }

    pub fn hash(&self) -> [u8; 32] {
        let mut hasher = sha2::Sha256::new();
        hasher.update(self.idx.to_be_bytes());
        hasher.update(self.timestamp.to_be_bytes());
        hasher.update(self.validator.as_bytes());
        hasher.update(self.parent_hash.as_bytes());
        hasher.update(self.merkle_root.as_bytes());
        hasher.finalize().into()
    }

    pub fn hash_str(&self) -> String {
        hex::encode(self.hash())
    }
}
