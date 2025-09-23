# Project initialization

---
We would use cargo workspaces for this project.
Using cargo workspaces is a powerful practice that greatly improves the Rust development experience when working with multiple related crates (libraries and executables).

## Main advantages:
### Single assembly and dependency caching. 
Within a workspace share a common target folder at the root level.
All dependencies are compiled once. 
If crate A and crate B use version 1.0 of the tokio library, it will only be downloaded and compiled once.

This significantly reduces build time and disk space. This is especially noticeable in large projects with many shared dependencies.
### Consistent versions of dependencies
Workspace makes it easy to ensure consistent versions of dependent libraries across all member crates.

You can define shared dependencies in the root Cargo.toml of a workspace under the `[workspace.dependencies]` section.

Workspace members can then reference these dependencies without specifying a version. Cargo ensures that all crates use the same library version.

### Build artifact caching. 
If you change code in only one crate, only that crate and the crates that depend on it are rebuilt. The remaining compiled artifacts remain in the cache.
Clear separation of responsibilities: You can create a crate library with the main logic (core_lib), a crate for the web API (web_api), a crate for the CLI (cli_tool), etc.

## Initializing
Create new cargo binary crate with `cargo new` command

```bash
cargo new blockchain && cd blockchain && rm -rf src
```

After this our project structure will look like this
```
blockchain/                # Workspace & .git root
├── Cargo.toml             # workspace configuration
├── target/                # target directory
├── .gitignore             # .gitignore
```

`Cargo.toml` example
```toml
[workspace]
resolver = "2"

[workspace.package]
version = "0.0.1"
repository = "https://github.com/daniilburavlev/rust-blockchain"
edition = "2024"

[workspace.dependencies]
```