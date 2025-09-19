# Local run 

---
## Building from source
```bash
git clone https://github.com/daniilburavlev/rust-blockchain.git && \
  cd rust-blockchain && \
  cargo build --release
```

## Generate local wallets for genesis block
```bash
./target/release/blockchain create
```

## Edit genesis block

You can edit `run/genesis.json` file and add transactions to your wallet, or mark wallets as validators for feature blocks

## Run local node

```bash
./target/release/blockchain start
```