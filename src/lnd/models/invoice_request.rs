use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LndInvoiceRequest {
    form: String,
}
impl LndInvoiceRequest {
    #[must_use]
    pub fn from_body(body: &LndInvoiceRequestBody) -> Self {
        Self {
            form: body.to_string(),
        }
    }
    #[must_use]
    pub fn new(amount: u64) -> Self {
        let body = LndInvoiceRequestBody {
            value: amount.to_string(),
            memo: None,
        };
        Self {
            form: body.to_string(),
        }
    }
}
impl std::fmt::Display for LndInvoiceRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", serde_json::to_string(self).unwrap_or_default())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LndInvoiceRequestBody {
    pub value: String,
    pub memo: Option<String>,
}
impl std::fmt::Display for LndInvoiceRequestBody {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", serde_json::to_string(self).unwrap_or_default())
    }
}
impl LndInvoiceRequestBody {
    #[must_use]
    pub const fn new(value: String, memo: Option<String>) -> Self {
        Self { value, memo }
    }
}
