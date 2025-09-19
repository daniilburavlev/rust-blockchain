use crate::chain::wallet::Wallet;
use crate::test::commons::{config, WALLET_PASSWORD};
use crate::test::commons::wallet;

#[test]
fn create_write_read_wallet() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config = config(temp_dir.path());
    let wallet = wallet(&config);
    let dir = config.keystore_path();
    wallet.write(&dir, WALLET_PASSWORD).unwrap();
    let restored = Wallet::read(&dir, &wallet.address(), WALLET_PASSWORD).unwrap();
    assert_eq!(wallet.address(), restored.address());
}
