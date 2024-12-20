use base64::prelude::*;
use serde::{Deserialize, Serialize};
use std::fmt::Display;

use super::OnchainAddressType;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum LndInvoiceState {
    #[serde(rename = "OPEN")]
    Open,
    #[serde(rename = "SETTLED")]
    Settled,
    #[serde(rename = "CANCELED")]
    Canceled,
    #[serde(rename = "ACCEPTED")]
    Accepted,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct LndPaymentInvoice {
    pub r_hash: String,
    pub payment_request: String,
    pub add_index: String,
    pub payment_addr: String,
}
impl TryFrom<String> for LndPaymentInvoice {
    type Error = anyhow::Error;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        Ok(serde_json::from_str(&value)?)
    }
}
impl TryInto<String> for LndPaymentInvoice {
    type Error = anyhow::Error;
    fn try_into(self) -> Result<String, Self::Error> {
        Ok(serde_json::to_string(&self)?)
    }
}
impl Display for LndPaymentInvoice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", serde_json::to_string_pretty(self).unwrap())
    }
}
impl LndPaymentInvoice {
    pub fn r_hash_url_safe(&self) -> String {
        let unsafe_str = BASE64_STANDARD.decode(&self.r_hash).unwrap();
        let url_safe = BASE64_URL_SAFE.encode(unsafe_str);
        url_safe
    }
    pub fn r_hash_hex(&self) -> String {
        let unsafe_str = BASE64_STANDARD.decode(&self.r_hash).unwrap();
        let hex = unsafe_str
            .iter()
            .map(|b| format!("{:02x}", b))
            .collect::<String>();
        hex
    }
    pub fn payment_hash(&self) -> Vec<u8> {
        BASE64_STANDARD.decode(&self.payment_addr).unwrap()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct LndInvoice {
    pub r_preimage: String,
    pub r_hash: String,
    pub payment_request: String,
    pub add_index: String,
    pub payment_addr: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memo: Option<String>,
    pub value: String,
    pub value_msat: String,
    pub settled: bool,
    pub creation_date: String,
    pub settle_date: String,
    pub state: LndInvoiceState,
}
impl TryFrom<String> for LndInvoice {
    type Error = anyhow::Error;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        Ok(serde_json::from_str(&value)?)
    }
}
impl TryInto<String> for LndInvoice {
    type Error = anyhow::Error;
    fn try_into(self) -> Result<String, Self::Error> {
        Ok(serde_json::to_string(&self)?)
    }
}
impl Display for LndInvoice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", serde_json::to_string_pretty(self).unwrap())
    }
}
impl LndInvoice {
    pub fn r_hash_url_safe(&self) -> String {
        let unsafe_str = BASE64_STANDARD.decode(&self.r_hash).unwrap();
        let url_safe = BASE64_URL_SAFE.encode(unsafe_str);
        url_safe
    }
    pub fn r_hash_hex(&self) -> String {
        let unsafe_str = BASE64_STANDARD.decode(&self.r_hash).unwrap();
        let hex = unsafe_str
            .iter()
            .map(|b| format!("{:02x}", b))
            .collect::<String>();
        hex
    }
    pub fn payment_hash(&self) -> Vec<u8> {
        BASE64_STANDARD.decode(&self.payment_addr).unwrap()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct LndInvoiceList {
    pub invoices: Vec<LndInvoice>,
    pub last_index_offset: String,
    pub first_index_offset: String,
}
impl TryFrom<String> for LndInvoiceList {
    type Error = anyhow::Error;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        Ok(serde_json::from_str(&value)?)
    }
}
impl Into<String> for LndInvoiceList {
    fn into(self) -> String {
        serde_json::to_string(&self).unwrap()
    }
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct LndNewAddress {
    pub addr: String,
}
impl TryFrom<String> for LndNewAddress {
    type Error = anyhow::Error;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        Ok(serde_json::from_str(&value)?)
    }
}
impl Into<String> for LndNewAddress {
    fn into(self) -> String {
        serde_json::to_string(&self).unwrap()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct LndNextAddressRequest {
    account: String,
    #[serde(rename = "type")]
    address_type: OnchainAddressType,
    change: bool,
}
impl TryFrom<String> for LndNextAddressRequest {
    type Error = anyhow::Error;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        Ok(serde_json::from_str(&value)?)
    }
}
impl Into<String> for LndNextAddressRequest {
    fn into(self) -> String {
        serde_json::to_string(&self).unwrap()
    }
}
impl Default for LndNextAddressRequest {
    fn default() -> Self {
        LndNextAddressRequest {
            account: "".to_string(),
            address_type: OnchainAddressType::TaprootPubkey,
            change: false,
        }
    }
}
