use crate::chain::tx::Tx;
use crate::test::commons::{config, wallet};

#[test]
fn test_new_tx() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config = config(temp_dir.path());
    let from = wallet(&config);
    let to = wallet(&config);
    let tx = Tx::new(&from, to.address(), String::from("0.0001"), 1).unwrap();
    assert!(tx.valid());
}
