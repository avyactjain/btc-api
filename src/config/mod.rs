use std::{fs, net::SocketAddr};

use serde::Deserialize;
use tracing::Level;

use crate::{btc_api_error::BtcApiError, chain::ChainName};

const DEFAULT_CONFIG_PATH: &str = "src/config/config.json";

#[derive(Deserialize, Debug)]
pub(crate) struct Config {
    #[serde(with = "socket_addr_serde")]
    pub listen_address: SocketAddr,
    pub chain_config: ChainConfig,

    #[serde(default = "default_log_level")]
    #[serde(with = "log_level_serde")]
    pub rust_log_level: Level,
    #[serde(default)]
    pub sign_txn: bool,
}

fn default_log_level() -> Level {
    Level::INFO
}

#[derive(Deserialize, Debug)]
pub(crate) struct ChainConfig {
    pub chain: ChainName,
    pub rpc_url: String,
    pub variant: ChainVariant,
}

#[derive(Deserialize, Debug, Clone, PartialEq, Eq)]
pub(crate) enum ChainVariant {
    Mainnet,
    Testnet,
}

// Custom serialization for SocketAddr since it doesn't implement Deserialize
mod socket_addr_serde {
    use serde::{self, Deserialize, Deserializer};
    use std::{net::SocketAddr, path::PathBuf};

    pub fn deserialize<'de, D>(deserializer: D) -> Result<SocketAddr, D::Error>
    where
        D: Deserializer<'de>,
    {
        let path = PathBuf::deserialize(deserializer)?;
        let addr_str = path.to_string_lossy();
        let socket_addr = addr_str.parse().map_err(serde::de::Error::custom)?;
        Ok(socket_addr)
    }
}

// Custom serialization for Level since it doesn't implement Deserialize
mod log_level_serde {
    use std::str::FromStr;

    use serde::{self, Deserialize, Deserializer};
    use tracing::Level;

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Level, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Level::from_str(&s).map_err(serde::de::Error::custom)
    }
}
impl Config {
    pub fn load() -> Result<Self, BtcApiError> {
        let config_str = fs::read_to_string(DEFAULT_CONFIG_PATH)
            .map_err(|e| BtcApiError::ConfigLoadError(e.to_string()))?;
        let config: Config = serde_json::from_str(&config_str)
            .map_err(|e| BtcApiError::ConfigLoadError(e.to_string()))?;
        Ok(config)
    }
}
