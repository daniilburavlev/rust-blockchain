use crate::blockchain::config::Config;
use std::fs;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;
use wallet::wallet::Wallet;

pub const WALLET_PASSWORD: &[u8] = b"password";

const KEYSTORE_PATH: &str = "/.keystore";
const VALIDATOR: &str = "/.keystore";
const STORAGE_PATH: &str = "/.storage";
const GENESIS_PATH: &str = "/genesis.json";
const PORT: i64 = 8080;

pub fn config(path: &Path) -> Config {
    let path = path.to_str().unwrap().to_string();
    Config::new(
        path.clone() + KEYSTORE_PATH,
        path.clone() + VALIDATOR,
        PORT,
        path.clone() + STORAGE_PATH,
        path.clone() + GENESIS_PATH,
        vec![]
    )
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
