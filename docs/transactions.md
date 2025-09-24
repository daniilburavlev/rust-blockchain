# Transactions
___
- [Concepts & Purpose](#concepts--purpose)
  - [Cryptographic hash function](#cryptographic-hash-function)
  - [Blockchain transaction](#blockchain-transaction)
  - [Cryptographic hash function](#signing-and-verification-of-transactions)
- [Implementation](#implementation)
- [Testing and usage](#testing-and-usage)

## Concepts & Purpose
___
### Cryptographic hash function
A cryptographic hash function is a one-way, fixed-length algorithm that transforms arbitrary-sized input data into a unique, fixed-size string called a hash value or digest.
In this project we will be using [SHA256](https://en.wikipedia.org/wiki/SHA-2) hash function.
### Blockchain transaction
#### Transaction
A blockchain transaction is the the transfer of a digital or physical assets, or data, between parties on a blockchain network, recorded as a block of data on a shared, immutable digital ledger.
Every transaction must be digitally signed by the sender account that authorizes of funds and authenticates of transaction.
Confirmed transactions are irreversible. 
Confirmed transactions are immutable. It is almost impossible to change the order or content of confirmed transactions 
#### Double spending problem
The situation when the same digital asset can be spent more then once. 
Only one of multiple transactions spending the same asset should be validated and confirmed while others transactions must be rejected. 
This blockchain prevents the double spending problem by tracking in the blockchain state both: the account balance to check for availability of funds, and the per-account monotonically increasing nonce to order transactions signed from the same account
### Signing and verification of transactions
#### Digital signature
The private signing key is used to produce a digital signature of a transaction. The corresponding public verifying key is used to verify the digital signature of a transaction. The digital signature proves the authenticity of a sender (origin authentication), the non-repudiation of a sender, and the integrity of a transaction (message authentication)
### Transaction search
Transactions stored locally on every node, transactions can be marked as verified by adding block index in their metadata.
Blockchain's members should be able to search for transactions by sender, recipient and block in which the transaction was confirmed. 
To achieve this, we implement indexes in our local storage
#### Sign transaction
#### Verify transaction
## Implementation
___
### Defining Tx structure
First at all we initialize new lib crate in our workspace
```bash
# blockchain/
cargo new chain
```

Update `Cargo.toml`
```toml
members = ["chain", "client", "node", "wallet"]
```

Now we need to add required dependencies `chain/Cargo.toml`
```toml
[dependencies]
libsecp256k1 = { workspace = true }
wallet = { path = "../wallet" }
bigdecimal = { workspace = true }
serde = { workspace = true }
sha2 = { workspace = true }
hex = { workspace = true }
```

```rust
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Tx {
    pub hash: String,
    pub from: String,
    pub to: String,
    pub amount: String,
    pub nonce: u64,
    pub timestamp: u64,
    pub signature: String,
    pub block: Option<u64>,
}
```

Next we define method to create new transaction
```rust
impl Tx {
  pub fn new(
    wallet: &Wallet,
    to: String,
    amount: String,
    nonce: u64,
  ) -> Result<Self, std::io::Error> {
    BigDecimal::from_str(amount.as_str())
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidInput, e.to_string()))?;
    // initializing main fields
    let mut tx = Self {
      hash: "".to_string(),
      from: wallet.address(),
      to,
      amount,
      nonce,
      timestamp: SystemTime::now()
              .duration_since(UNIX_EPOCH)
              .unwrap()
              .as_secs(),
      signature: "".to_string(),
      block: None,
    };
    // signing by sender
    let signature = wallet.sign(&tx.hash())?;
    tx.signature = signature;
    tx.hash = tx.hash_str();
    Ok(tx)
  }
}
```

Our hash function will contain address from, address to, amount, nonce and timestamp
```rust
impl Tx {
  ...
  pub fn hash(&self) -> [u8; 32] {
    let mut hasher = sha2::Sha256::new();
    hasher.update(self.from.as_bytes());
    hasher.update(self.to.as_bytes());
    hasher.update(self.amount.as_bytes());
    hasher.update(self.nonce.to_be_bytes());
    hasher.update(self.timestamp.to_be_bytes());
    hasher.finalize().into()
  }
  // string representation
  pub fn hash_str(&self) -> String {
    let hash = self.hash();
    hex::encode(&hash)
  }
  ...
}
```

Verification method
```rust
impl Tx { 
  pub fn valid(&self) -> bool {
    // check amount validity
    if BigDecimal::from_str(self.amount.as_str()).is_err() {
        return false;
    }
    match hex::decode(&self.from) {
      Ok(key_bytes) => {
        if key_bytes.len() != 33 {
            return false;
        }
        // check key validity
        let key_bytes: [u8; 33] = key_bytes.try_into().unwrap();
        // parsing public key
        match libsecp256k1::PublicKey::parse_compressed(&key_bytes) {
          Ok(public_key) => match hex::decode(&self.signature) {
            Ok(signature) => {
              if signature.len() != 64 {
                return false;
              }
              let signature_bytes: [u8; 64] = signature.try_into().unwrap();
              match libsecp256k1::Signature::parse_standard(&signature_bytes) {
                Ok(signature) => libsecp256k1::verify(
                    &libsecp256k1::Message::parse(&self.hash()),
                    &signature,
                    &public_key,
                ),
                Err(_) => false,
              }
            }
            Err(_) => false,
          },
          Err(_) => false,
        }
      }
      Err(_) => false,
    }
  }
}
```
## Testing and usage
___
- Create from wallet
- Create to wallet (optional, only need for valid address)
- Create new tx
- Check valid method returns true
```rust
#[test]
fn test_new_tx() {
  let from = Wallet::new();
  let to = Wallet::new();
  let tx = Tx::new(&from, to.address(), String::from("0.0001"), 1).unwrap();
  assert!(tx.valid());
} 
```