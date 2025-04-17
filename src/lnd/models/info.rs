use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fmt::Display};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chain {
    pub chain: String,
    pub network: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Feature {
    pub name: String,
    pub is_required: bool,
    pub is_known: bool,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeaturesEntry {
    pub key: u32,
    pub value: Feature,
}
#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LndInfo {
    pub version: String,
    pub commit_hash: String,
    pub identity_pubkey: String,
    pub alias: String,
    pub color: String,
    pub num_pending_channels: u32,
    pub num_active_channels: u32,
    pub num_inactive_channels: u32,
    pub num_peers: u32,
    pub block_height: u32,
    pub block_hash: String,
    pub best_header_timestamp: String,
    pub synced_to_chain: bool,
    pub synced_to_graph: bool,
    pub testnet: bool,
    pub chains: Vec<Chain>,
    pub uris: Vec<String>,
    pub features: HashMap<String, Feature>,
    pub require_htlc_interceptor: bool,
    pub store_final_htlc_resolutions: bool,
}

impl TryFrom<&str> for LndInfo {
    type Error = anyhow::Error;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Ok(serde_json::from_str(value)?)
    }
}
impl TryFrom<&String> for LndInfo {
    type Error = anyhow::Error;
    fn try_from(value: &String) -> Result<Self, Self::Error> {
        Ok(serde_json::from_str(value)?)
    }
}
impl TryFrom<String> for LndInfo {
    type Error = anyhow::Error;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        Ok(serde_json::from_str(&value)?)
    }
}
impl Display for LndInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", serde_json::to_string_pretty(self).unwrap())
    }
}
