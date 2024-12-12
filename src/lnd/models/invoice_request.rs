use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LndInvoiceRequest {
    form: String,
}
impl LndInvoiceRequest {
    pub fn from_body(body: LndInvoiceRequestBody) -> Self {
        Self {
            form: body.to_string(),
        }
    }
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
impl ToString for LndInvoiceRequest {
    fn to_string(&self) -> String {
        serde_json::to_string(self).unwrap()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LndInvoiceRequestBody {
    pub value: String,
    pub memo: Option<String>,
}
impl Default for LndInvoiceRequestBody {
    fn default() -> Self {
        Self {
            value: "".to_string(),
            memo: None,
        }
    }
}
impl ToString for LndInvoiceRequestBody {
    fn to_string(&self) -> String {
        serde_json::to_string(self).unwrap()
    }
}
impl LndInvoiceRequestBody {
    pub fn new(value: String, memo: Option<String>) -> Self {
        Self { value, memo }
    }
}
