use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LndListAddressesResponse {
    pub account_with_addresses: Vec<AccountWithAddresses>,
}
impl LndListAddressesResponse {
    pub fn find_default_addresses(&self) -> Vec<LndAddressProperty> {
        self.account_with_addresses
            .iter()
            .find(|account| account.name == "default")
            .unwrap()
            .addresses
            .clone()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountWithAddresses {
    pub name: String,
    pub address_type: OnchainAddressType,
    pub derivation_path: String,
    pub addresses: Vec<LndAddressProperty>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OnchainAddressType {
    #[serde(rename = "UNKNOWN")]
    Unknown = 0,
    #[serde(rename = "WITNESS_PUBKEY_HASH")]
    WitnessPubkeyHash = 1,
    #[serde(rename = "NESTED_WITNESS_PUBKEY_HASH")]
    NestedWitnessPubkeyHash = 2,
    #[serde(rename = "HYBRID_NESTED_WITNESS_PUBKEY_HASH")]
    HybridNestedWitnessPubkeyHash = 3,
    #[serde(rename = "TAPROOT_PUBKEY")]
    TaprootPubkey = 4,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LndAddressProperty {
    pub address: String,
    pub is_internal: bool,
    pub balance: String,
    pub derivation_path: String,
    pub public_key: String,
}
