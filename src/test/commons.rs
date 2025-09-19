use crate::chain::config::Config;
use crate::chain::wallet::Wallet;
use std::fs;
use std::fs::OpenOptions;
use std::io::Write;

pub const WALLET_PASSWORD: &[u8] = b"password";

pub fn config() -> Config {
    Config::from_file("test/config.json").unwrap()
}

pub fn wallet(config: &Config) -> Wallet {
    let wallet = Wallet::new();
    wallet
        .write(&config.keystore_path(), WALLET_PASSWORD)
        .unwrap();
    wallet
}

pub fn wallet_with_balance(config: &Config) -> Result<Wallet, std::io::Error> {
    fs::create_dir_all(&config.keystore_path())?;
    let wallet = Wallet::new();
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(config.genesis_path())?;
    let json = format!(
        "[{{\"hash\": \"GENESIS_133ec3db684243afafa83055a5f69a65\",\"from\": \"GENESIS\",\"to\": \"{}\",\"amount\": \"1000000\",\"nonce\": 1,\"timestamp\": 1009227600,\"signature\": \"GENESIS\",\"block\": 0}}, \
        {{\"hash\": \"GENESIS_52a2476f72e3491d88c7e82d5aa52469\",\"from\": \"{}\",\"to\": \"STAKE\",\"amount\": \"500000\",\"nonce\": 1,\"timestamp\": 1009227600,\"signature\": \"GENESIS\",\"block\": 0}}]",
        wallet.address(),
        wallet.address()
    );
    file.write(json.as_bytes())?;
    Ok(wallet)
}
