# Local run 

---
## Building from source
```bash
git clone https://github.com/daniilburavlev/rust-blockchain.git && \
  cd rust-blockchain && \
  cargo build --release
```

## Creating wallets
You can generate local wallets using command below
```bash
./target/release/node create
```

### Genesis block
Genesis block data stored in `run/genesis.json` file, you can edit it to include your wallets or stakes

#### Adding to balance after genesis block mined:
```json
[
  {
    "hash": "GENESIS_519c2c360cbd4f0a8b6842fa9556b9e0",
    "from": "GENESIS",
    "to": "034e363531822d1eac09910d7e6fb7fff4b6df9278c297b516c9f91e9faecbb5bb",
    "amount": "2653090",
    "nonce": 1,
    "timestamp": 1009227600,
    "signature": "GENESIS",
    "block": 0
  }
]
```

#### Stake value
```json
[
  {
    "hash": "GENESIS_519c2c360cbd4f0a8b6842fa9556b9e0",
    "from": "034e363531822d1eac09910d7e6fb7fff4b6df9278c297b516c9f91e9faecbb5bb",
    "to": "STAKE",
    "amount": "1323090",
    "nonce": 1,
    "timestamp": 1009227600,
    "signature": "GENESIS",
    "block": 0
  }
]
```

## Run local node

```bash
./target/release/node start
```
after launching in the console you can see the address where the service is available

```
Node started: /ip4/127.0.0.1/tcp/8089
Node started: /ip4/127.0.0.1/udp/8089/quic-v1
```

## Configure other nodes

### Example:
```json
{
  "validator": "other_wallet_address",
  "port": 0, // for random port
  "keystore_path": "path_to_keystore",
  "storage_path": "path_to_storage",
  "genesis_path": "run/genesis.json",
  "nodes" : [
  ]
}
```

After running nodes will be synced and ready to communicate

## Create new transaction
```bash
./target/release/node tx --from wallet_from \
  --to wallet-to --amount 10
```

#### Output:
```
Transaction successfully submitted
```
