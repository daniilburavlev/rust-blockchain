use std::fs;
use std::sync::Arc;
use crate::chain::storage::db;
use crate::chain::storage::nonce_storage::NonceStorage;
use crate::test::commons::config;

#[test]
fn test_nonce_save_get() -> Result<(), std::io::Error> {
    let temp_dir = tempfile::tempdir()?;
    let config = config(temp_dir.path());
    let db = db::open(&config)?;
    let nonce_storage = NonceStorage::new(Arc::clone(&db));;
    let value = 90;
    nonce_storage.save(String::from("wallet"), value)?;
    let found = nonce_storage.get(String::from("wallet"))?;
    assert_eq!(value, found);
    let zero = nonce_storage.get(String::from("wallet1"))?;
    assert_eq!(zero, 0);
    fs::remove_dir_all(config.storage_path())?;
    Ok(())
}
