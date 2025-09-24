# Build Proof-of-Stake blockchain from scratch in Rust with libp2p

---
Note: This guide and project is inspired by [volodymyrprokopyuk](https://github.com/volodymyrprokopyuk)'s [guide](https://github.com/volodymyrprokopyuk/go-blockchain). If you found some mistakes in code or in guide feel free to open issues or merge request!

## Preview

---
The rise of blockchain technology demands robust, secure, and high-performance implementations. 
While many blockchains were pioneered using languages like C++ and Go, the unique advantages of the Rust programming language make it an increasingly compelling choice for building the next generation of decentralized systems. 
This article explores the implementation of a core blockchain in Rust, leveraging its powerful features like memory safety without garbage collection, zero-cost abstractions, and fearless concurrency to create a reliable and efficient distributed ledger.

## Contents

### [Installation & Usage](docs/local_run.md)

---
### [Project initialization](docs/project_initialization.md)
Configuring local dev, cargo workspaces initialization

### [Wallets](docs/wallet.md)
Main information about blockchain wallets, private/public keys, signing, persisting wallet locally

### [Transactions](docs/transactions.md)
Creating new transactions, signing transactions, double spending problem.