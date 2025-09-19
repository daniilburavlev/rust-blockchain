use serde::{Deserialize, Serialize};
use std::fs;

pub const DEFAULT_CONFIG_PATH: &str = "run/config.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    keystore_path: String,
    validator: String,
    port: i64,
    storage_path: String,
    genesis_path: String,
    nodes: Vec<String>,
}

impl Config {
    pub fn new(
        keystore_path: String,
        validator: String,
        port: i64,
        storage_path: String,
        genesis_path: String,
        nodes: Vec<String>,
    ) -> Self {
        Self {
            keystore_path,
            validator,
            port,
            storage_path,
            genesis_path,
            nodes,
        }
    }

    pub fn from_file(path: &str) -> Result<Self, std::io::Error> {
        let json = fs::read_to_string(path)?;
        let config: Config = serde_json::from_str(&json)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))?;
        Ok(config)
    }

    pub fn port(&self) -> i64 {
        self.port
    }

    pub fn genesis_path(&self) -> String {
        self.genesis_path.clone()
    }

    pub fn validator(&self) -> String {
        self.validator.clone()
    }

    pub fn storage_path(&self) -> String {
        self.storage_path.clone()
    }

    pub fn keystore_path(&self) -> String {
        self.keystore_path.clone()
    }

    pub fn nodes(&self) -> Vec<String> {
        self.nodes.clone()
    }
}
