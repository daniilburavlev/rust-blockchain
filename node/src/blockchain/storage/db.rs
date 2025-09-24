use std::sync::Arc;
use crate::blockchain::config;
use rocksdb::{DBWithThreadMode, MultiThreaded, Options};

pub fn open(config: &config::Config) -> Result<Arc<DBWithThreadMode<MultiThreaded>>, std::io::Error> {
    let mut options = Options::default();
    options.create_if_missing(true);
    let db = DBWithThreadMode::open(&options, config.storage_path())
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
    Ok(Arc::new(db))
}
