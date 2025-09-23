# Wallet
- [Concepts and purpose](#concepts-and-purpose)
  - [Blockchain wallet](#blockchain-wallet)
- [Design & Implementation](#design-and-implementation)
  - [Secp256k1 key pair](#secp256k1-key-pair)
  - [Wallet implementation](#wallet-implementation)
  - [Persisting wallet](#persisting-wallet)
- [Testing & Usage](#testing-and-usage)
---
## Concepts and purpose

---
### Blockchain wallet
The blockchain wallet is a digital identity on the blockchain. It represents user’s ownership of assets on the blockchain. The blockchain wallet can hold, send, and receive cryptocurrency. 

#### *Private key*
The secret is a randomly generated number known as the private key.

#### *Wallet address*
The wallet address is a hex decoded value of a public key, derived from a corresponding private key, that uniquely identifies the wallet on the blockchain. 

#### *Wallet balance*
The wallet balance is the amount of divisible cryptocurrency or indivisible tokens controlled by the private key of the account

#### *Wallet transactions*
The wallet transactions is a time-ordered list of debit and credit transactions involving the account



## Design and implementation

---
### Secp256k1 key pair
The implementation of this blockchain uses the Elliptic-Curve Cryptography (ECC). Specifically, the Secp256k1 elliptic curve is used for generation of key pairs for accounts on the blockchain, as well as signing and verification of transactions on the blockchain

#### *Private key*
The private key is a large randomly generated secret number that is used to derive the public key and digitally sign transactions. The private key must be kept in secret to preserve account authenticity and control account assets on the blockchain

#### *Public key*
The public key is a pair of large numbers derived from the private key. The public key is used to identify the account on the blockchain and verify transactions signed with the account private key. The public key can be safely shared with any participant on the blockchain

### Wallet implementation

Firstly we create new cargo binary crate in workspace

```bash
cd blockchain && cargo new wallet
```

Edit workspace's `Cargo.toml`
```toml
[workspace]
resolver = "2"
# adding wallet as new memeber
members = ["wallet"]
...
```
Now we will work with wallet lib `cd wallet`

Adding new mod `src/chain/mod.rs`
```rust
pub mod wallet;
```

`lib.rs` file
```rust
pub mod wallet;
mod crypto;
#[cfg(test)]
mod wallet_test;
```
Adding required dependencies for wallet in main `Cargo.toml`
```toml
["workspace.dependencies"]
argon2 = "0.5.3"
libsecp256k1 = "0.7.2"
rand = "0.8"
```
Wallet's `Cargo.toml`
```toml
["dependencies"]
argon2 = { workspace = true }
libsecp256k1 = { workspace = true }
rand = { workspace = true }
```

Define wallet structure in `src/chain/wallet.rs`
```rust
#[derive(Clone, Debug)]
pub struct Wallet {
    secret: [u8; 32],
    address: [u8; 33],
}
```

Initialization method
```rust
impl Wallet {
    pub fn new() -> Self {
        // Init secp256k1 private key
        let secret = libsecp256k1::SecretKey::random(&mut rand::rngs::OsRng);
        // Get public key from private
        let public = libsecp256k1::PublicKey::from_secret_key(&secret);
        Wallet {
            secret: secret.serialize(),
            address: public.serialize_compressed(),
        }
    }
    ...
}
```

We also define getter methods

```rust
impl Wallet {
    ...
    pub fn secret(&self) -> [u8; 32] {
        self.secret.clone()
    }

    pub fn address(&self) -> String {
        hex::encode(self.address)
    }
    ...
}
```
### Persisting wallet
The private key is the only piece of information required to re-create an account after persisting the account to an encrypted file protected with the owner-provided password. 
Wallets on this example are persisted locally to files with restricted access. 
The encoded key pair of the account is encrypted with the owner-provided password before being persisted to a file with restricted access. 
Only the owner of the account can re-create the account and use the account to sign transactions by providing the correct password to decrypt the account key pair

#### Algorithm

- Generate salt and key from owner-provided password with argon2
- Encrypt the wallet's secret with the generated key
- Write encrypted salt, nonce and encrypted secret into local file

#### Implementation

```rust
// wallet/src/crypto.rs

pub fn derive_key(password: &[u8]) -> Result<([u8; 16], [u8; 32]), std::io::Error> {
  // Generating password random salt
  let salt = argon2::password_hash::SaltString::generate(&mut rand::rngs::OsRng);
  // Getting argon instance
  let argon2 = argon2::Argon2::default();
  // Hashing password
  let password_hash = argon2
          .hash_password(password, &salt)
          .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))?
          .hash
          .unwrap();
  if password_hash.as_bytes().len() != 32 {
    return Err(std::io::Error::new(
      std::io::ErrorKind::InvalidInput,
      "Invalid password hash",
    ));
  }
  let mut key_bytes = [0u8; 32];
  // Generating key from hashed password
  key_bytes.copy_from_slice(&password_hash.as_bytes());
  let mut salt_bytes = [0u8; 16];
  salt.decode_b64(&mut salt_bytes).unwrap();
  Ok((salt_bytes, key_bytes))
}
```

Encrypt function 
```rust
// wallet/src/crypto.rs
pub fn encrypt_data(key: &[u8], data: &[u8]) -> Result<(Vec<u8>, [u8; 12]), std::io::Error> {
    let cipher = aes_gcm::Aes256Gcm::new(key.into());
    let mut nonce_bytes = [0u8; 12];
    rand::rngs::OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = aes_gcm::Nonce::from_slice(&nonce_bytes);
    let cipher_data = cipher
        .encrypt(nonce, data)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))?;
    Ok((cipher_data, nonce_bytes))
}
```

```rust
// wallet/src/wallet.rs
impl Wallet {
  ...
  pub fn write(&self, dir: &str, password: &[u8]) -> Result<(), std::io::Error> {
    fs::create_dir_all(dir)?;
    let (salt, key) = crypto::derive_key(password)?;
    let (data, nonce) = crypto::encrypt_data(&key, &self.secret)?;
    let mut file = OpenOptions::new().write(true).create(true).open(format!(
      "{}/{}",
      dir,
      hex::encode(self.address)
    ))?;
    file.write_all(&salt)?;
    file.write_all(&nonce)?;
    file.write_all(data.as_slice())?;
    Ok(())
  }
  ...
}
```

### Wallet recreation
#### Algorithm

- Read salt, nonce and encrypted data
- Restore key from salt and owner-provided password
- Decrypt secret with restored key
- Recreate wallet with secret

#### Implementation
Restoring key

```rust
// wallet/src/crypto.rs
pub fn restore_key(salt: &[u8], password: &[u8]) -> Result<[u8; 32], std::io::Error> {
  let argon2 = argon2::Argon2::default();
  let salt = argon2::password_hash::SaltString::encode_b64(salt)
          .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidInput, e.to_string()))?;
  let password_hash = argon2.hash_password(password, &salt).unwrap().hash.unwrap();
  if password_hash.as_bytes().len() != 32 {
    return Err(std::io::Error::new(
      std::io::ErrorKind::InvalidData,
      "Invalid password hash",
    ));
  }
  let mut key_bytes = [0u8; 32];
  key_bytes.copy_from_slice(password_hash.as_bytes());
  Ok(key_bytes)
}
```

Decrypt function
```rust
// wallet/src/crypto.rs
pub fn decrypt_data(
    key: &[u8],
    data: &[u8],
    nonce_bytes: &[u8],
) -> Result<Vec<u8>, std::io::Error> {
    let cipher = aes_gcm::Aes256Gcm::new(key.into());
    let nonce = aes_gcm::Nonce::from_slice(nonce_bytes);
    let text = cipher
        .decrypt(nonce, data)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))?;
    Ok(text)
}
```

Reading wallet data from file

```rust
impl Wallet {
  ...
  pub fn read(dir: &str, address: &str, password: &[u8]) -> Result<Self, std::io::Error> {
    let path = format!("{}/{}", dir, address);
    let mut file = File::open(path)?;
    let mut salt = [0u8; 16];
    let mut nonce = [0u8; 12];
    let mut data = Vec::new();
    file.read_exact(&mut salt)?;
    file.read_exact(&mut nonce)?;
    file.read_to_end(&mut data)?;
    let key = crypto::restore_key(&salt, password)?;
    let secret = crypto::decrypt_data(&key, data.as_slice(), &nonce)?;
    let secret: [u8; 32] = secret[..].try_into().unwrap();
    Self::from_secret(secret)
  }
  ...
}
```

Restoring wallet from secret

```rust
impl Wallet {
  ...
  pub fn from_secret(secret: [u8; 32]) -> Result<Self, std::io::Error> {
    let secret = libsecp256k1::SecretKey::parse(&secret)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidInput, e.to_string()))?;
    let public = libsecp256k1::PublicKey::from_secret_key(&secret);
    Ok(Wallet {
      secret: secret.serialize(),
      address: public.serialize_compressed(),
    })
  }
  ...
}
```

## Testing and usage

---
### Testing wallet saving and re-creation
By default, tests in cargo are multithreaded. 
When working with files, we must take this fact. To prevent situation then different threads changing one file, we will use crate `tempfile`, 
which will also allow us to automatically clear data after the test is completed.

Add tempfile to `Cargo.toml` in `dev-dependencies` to exclude it from release executable
```toml
[dev-dependencies]
tempfile = "3.22.0"
```

- Create tempdir for saving wallet
- Create wallet
- Write wallet with password
- Restore wallet
- Check restores addresses are equal

```rust
#[test]
fn create_write_read_wallet() {
    let temp_dir = tempfile::tempdir().unwrap();
    let wallet = Wallet::new();
    wallet.write(&dir, WALLET_PASSWORD).unwrap();
    let restored = Wallet::read(temp_dir.path().as_str(), &wallet.address(), WALLET_PASSWORD).unwrap();
    assert_eq!(wallet.address(), restored.address());
}
```
