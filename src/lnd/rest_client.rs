use std::io::Read;

use base64::prelude::*;

use crate::{
    lnd::{LndHodlInvoice, LndHodlInvoiceState, LndInfo, LndInvoice, LndInvoiceRequestBody},
    LndInvoiceList, LndWebsocket,
};
use reqwest::header::{HeaderMap, HeaderValue};

use super::{
    LndAddressProperty, LndListAddressesResponse, LndNewAddress, LndNextAddressRequest,
    LndPaymentInvoice, OnchainAddressType,
};

#[derive(Clone)]
pub struct LightningClient {
    url: &'static str,
    data_dir: &'static str,
    pub client: reqwest::Client,
}

impl LightningClient {
    pub async fn dud_server() -> anyhow::Result<Self> {
        let client = reqwest::Client::builder()
            .danger_accept_invalid_certs(true)
            .build()?;
        Ok(Self {
            url: "localhost:10009",
            client,
            data_dir: "",
        })
    }
    pub async fn new(url: &'static str, data_dir: &'static str) -> anyhow::Result<Self> {
        let mut default_header = HeaderMap::new();
        let macaroon = Self::macaroon(data_dir)?;
        let mut header_value = HeaderValue::from_str(&macaroon).unwrap();
        header_value.set_sensitive(true);
        default_header.insert("Grpc-Metadata-macaroon", header_value);
        default_header.insert("Accept", HeaderValue::from_static("application/json"));
        default_header.insert("Content-Type", HeaderValue::from_static("application/json"));
        let client = reqwest::Client::builder()
            .danger_accept_invalid_certs(true)
            .default_headers(default_header)
            .build()?;
        Ok(Self {
            url,
            client,
            data_dir,
        })
    }
    fn macaroon(data_dir: &'static str) -> anyhow::Result<String> {
        let mut macaroon = vec![];
        let mut file = std::fs::File::open(data_dir)?;
        file.read_to_end(&mut macaroon)?;
        Ok(macaroon.iter().map(|b| format!("{:02x}", b)).collect())
    }
    pub async fn get_info(&self) -> anyhow::Result<LndInfo> {
        let url = format!("https://{}/v1/getinfo", self.url);
        let response = self.client.get(&url).send().await?;
        let response = response.text().await?;
        LndInfo::try_from(response)
    }
    pub async fn channel_balance(&self) -> anyhow::Result<()> {
        let url = format!("https://{}/v1/balance/channels", self.url);
        let response = self.client.get(&url).send().await?;
        let _response = response.text().await?;
        Ok(())
    }
    pub async fn get_invoice(
        &self,
        form: LndInvoiceRequestBody,
    ) -> anyhow::Result<LndPaymentInvoice> {
        let url = format!("https://{}/v1/invoices", self.url);
        let response = self.client.post(&url).body(form.to_string());
        let response = response.send().await?;
        let response = response.json::<LndPaymentInvoice>().await?;
        Ok(response)
    }
    pub async fn list_invoices(&self) -> anyhow::Result<Vec<LndInvoice>> {
        let url = format!("https://{}/v1/invoices", self.url);
        let response = self.client.get(&url).send().await?;
        let response = response.json::<LndInvoiceList>().await?;
        Ok(response.invoices)
    }
    pub async fn new_onchain_address(
        &self,
        request: LndNextAddressRequest,
    ) -> anyhow::Result<LndNewAddress> {
        let url = format!("https://{}/v2/wallet/address/next", self.url);
        let request_str: String = request.into();
        let response = self.client.post(&url).body(request_str).send().await?;
        tracing::info!("{:?}", response);
        let response = response.json::<LndNewAddress>().await?;
        Ok(response)
    }
    pub async fn list_onchain_addresses(
        &self,
        account: &str,
        address_type: OnchainAddressType,
    ) -> anyhow::Result<Vec<LndAddressProperty>> {
        let url = format!("https://{}/v2/wallet/addresses", self.url);
        let response = self.client.get(&url).send().await?;
        let response = response
            .json::<LndListAddressesResponse>()
            .await?
            .find_addresses(account, address_type);
        Ok(response)
    }
    pub async fn invoice_channel(&self) -> anyhow::Result<LndWebsocket> {
        let url = format!("wss://{}/v2/router/send?method=POST", self.url);
        let lnd_ws =
            LndWebsocket::new(self.url.to_string(), Self::macaroon(self.data_dir)?, url).await?;
        Ok(lnd_ws)
    }
    pub async fn lookup_invoice(
        &self,
        r_hash_url_safe: String,
    ) -> anyhow::Result<LndHodlInvoiceState> {
        let query = format!(
            "https://{}/v2/invoices/lookup?payment_hash={}",
            self.url, r_hash_url_safe
        );
        let response = self.client.get(&query).send().await?;
        let response = response.json::<LndHodlInvoiceState>().await?;
        Ok(response)
    }
    pub async fn subscribe_to_invoice(
        &self,
        r_hash_url_safe: String,
    ) -> anyhow::Result<LndWebsocket> {
        let query = format!(
            "wss://{}/v2/invoices/subscribe/{}",
            self.url, r_hash_url_safe
        );
        let lnd_ws =
            LndWebsocket::new(self.url.to_string(), Self::macaroon(self.data_dir)?, query).await?;
        Ok(lnd_ws)
    }
    pub async fn get_hodl_invoice(
        &self,
        payment_hash: String,
        amount: u64,
    ) -> anyhow::Result<LndHodlInvoice> {
        let url = format!("https://{}/v2/invoices/hodl", self.url);

        let response = self
            .client
            .post(&url)
            .json(&serde_json::json!({ "value": amount, "hash": payment_hash }))
            .send()
            .await?;
        let response = response.text().await?;
        LndHodlInvoice::try_from(response)
    }
    pub async fn settle_htlc(&self, preimage: String) -> anyhow::Result<()> {
        let url = format!("https://{}/v2/invoices/settle", self.url);
        let hex_bytes = preimage.chars().collect::<Vec<char>>();
        let preimage = hex_bytes
            .chunks(2)
            .map(|chunk| {
                let s: String = chunk.iter().collect();
                u8::from_str_radix(&s, 16).unwrap()
            })
            .collect::<Vec<u8>>();
        let preimage = BASE64_URL_SAFE.encode(&preimage);
        let response = self
            .client
            .post(&url)
            .json(&serde_json::json!({ "preimage": preimage }))
            .send()
            .await?;
        let _test = response.text().await?;
        Ok(())
    }
    pub async fn cancel_htlc(&self, payment_hash: String) -> anyhow::Result<()> {
        let url = format!("https://{}/v2/invoices/cancel", self.url);
        let response = self
            .client
            .post(&url)
            .json(&serde_json::json!({ "payment_hash": payment_hash }))
            .send()
            .await?;
        response.text().await?;
        Ok(())
    }
}

#[cfg(test)]
mod test {

    use crate::{
        lnd::HodlState, InvoicePaymentState, LightningAddress, LndHodlInvoiceState, LndInvoice,
        LndInvoiceRequestBody, LndInvoiceState, LndNextAddressRequest, LndPaymentRequest,
        LndPaymentResponse, LndWebsocketMessage,
    };
    use futures_util::StreamExt;
    use tracing::{error, info};
    use tracing_test::traced_test;

    use super::LightningClient;
    #[tokio::test]
    #[traced_test]
    async fn next_onchain() -> anyhow::Result<()> {
        let client = LightningClient::new("lnd.illuminodes.com", "./admin.macaroon").await?;
        let invoices = client
            .new_onchain_address(LndNextAddressRequest::default())
            .await?;

        info!("{:?}", invoices);
        Ok(())
    }
    #[tokio::test]
    #[traced_test]
    async fn onchain_list() -> anyhow::Result<()> {
        let client = LightningClient::new("lnd.illuminodes.com", "./admin.macaroon").await?;
        let invoices = client
            .list_onchain_addresses("default", crate::OnchainAddressType::TaprootPubkey)
            .await?;
        info!("{:?}", invoices);
        Ok(())
    }

    #[tokio::test]
    #[traced_test]
    async fn test_invoice_list() -> anyhow::Result<()> {
        let client = LightningClient::new("lnd.illuminodes.com", "./admin.macaroon").await?;
        let invoices = client.list_invoices().await?;
        info!("{:?}", invoices);
        Ok(())
    }
    #[tokio::test]
    #[traced_test]
    async fn test_connection() -> anyhow::Result<()> {
        let client = LightningClient::new("lnd.illuminodes.com", "./admin.macaroon").await?;
        let invoice = client
            .get_invoice(LndInvoiceRequestBody {
                value: 1000.to_string(),
                memo: Some("Hello".to_string()),
                ..Default::default()
            })
            .await?;
        info!("{:?}", invoice);
        let mut subscription = client
            .subscribe_to_invoice(invoice.r_hash_url_safe())
            .await?;
        loop {
            match subscription.event_stream::<LndInvoice>().next().await {
                Some(LndWebsocketMessage::Response(state)) => {
                    info!("{:?}", state);
                    match state.state {
                        LndInvoiceState::Open => {
                            break;
                        }
                        LndInvoiceState::Canceled => {
                            break;
                        }
                        _ => {}
                    }
                }
                Some(LndWebsocketMessage::Error(e)) => {
                    tracing::error!("{}", e);
                    Err(anyhow::anyhow!("Error"))?;
                }
                Some(LndWebsocketMessage::Ping) => {
                    info!("Ping");
                }
                None => {
                    Err(anyhow::anyhow!("No state"))?;
                }
            }
        }
        Ok(())
    }
    #[tokio::test]
    #[traced_test]
    async fn get_hodl_invoice() -> anyhow::Result<()> {
        let client = LightningClient::new("lnd.illuminodes.com", "./admin.macaroon").await?;
        let ln_address = LightningAddress("42pupusas@blink.sv");
        let pay_request = ln_address.get_invoice(&client.client, 1000).await?;
        let _hodl_invoice = client.get_hodl_invoice(pay_request.r_hash()?, 100).await?;
        let mut states = client
            .subscribe_to_invoice(pay_request.r_hash_url_safe()?)
            .await?;
        let mut correct_state = false;
        while let Some(LndWebsocketMessage::Response(state)) =
            states.event_stream::<LndHodlInvoiceState>().next().await
        {
            info!("{:?}", state.state());
            match state.state() {
                HodlState::OPEN => {
                    client.cancel_htlc(pay_request.r_hash_url_safe()?).await?;
                }
                HodlState::CANCELED => {
                    correct_state = true;
                    break;
                }
                _ => {}
            }
        }
        assert!(correct_state);
        Ok(())
    }

    #[tokio::test]
    #[traced_test]
    async fn pay_invoice() -> anyhow::Result<()> {
        let client = LightningClient::new("lnd.illuminodes.com", "./admin.macaroon").await?;
        let ln_address = "42pupusas@blink.sv";
        let pay_request = LightningAddress(ln_address)
            .get_invoice(&client.client, 100000)
            .await?;
        let pr = LndPaymentRequest::new(pay_request.pr.clone(), 10, 10.to_string(), false);
        let mut lnd_ws = client.invoice_channel().await?;
        let mut receiver = lnd_ws.event_stream::<LndPaymentResponse>();
        lnd_ws.sender.send(pr.clone()).await.unwrap();
        while let Some(LndWebsocketMessage::Response(state)) = receiver.next().await {
            match state.status() {
                InvoicePaymentState::Initiaited => {
                    info!("Initiated");
                }
                InvoicePaymentState::InFlight => {
                    info!("InFlight");
                }
                InvoicePaymentState::Succeeded => {
                    info!("Succeeded");
                    break;
                }
                InvoicePaymentState::Failed => {
                    error!("Failed");
                    break;
                }
            }
        }
        Ok(())
    }
    #[tokio::test]
    #[traced_test]
    async fn settle_htlc() -> Result<(), anyhow::Error> {
        use std::sync::Arc;
        use tokio::sync::Mutex;
        let client = LightningClient::new("lnd.illuminodes.com", "./admin.macaroon").await?;
        let ln_address = "42pupusas@blink.sv";
        let pay_request = LightningAddress(ln_address)
            .get_invoice(&client.client, 100000)
            .await?;

        let hodl_invoice = client.get_hodl_invoice(pay_request.r_hash()?, 20).await?;
        info!("{:?}", hodl_invoice.payment_request());
        let correct_state = Arc::new(Mutex::new(false));
        let mut states = client
            .subscribe_to_invoice(hodl_invoice.r_hash_url_safe()?)
            .await?
            .event_stream::<LndHodlInvoiceState>();

        let pr = LndPaymentRequest::new(pay_request.pr.clone(), 1000, 10.to_string(), false);
        let mut lnd_ws = client.invoice_channel().await?;
        let mut receiver = lnd_ws.event_stream::<LndPaymentResponse>();
        tokio::spawn(async move {
            loop {
                match receiver.next().await {
                    Some(LndWebsocketMessage::Response(state)) => {
                        info!("Listening for payment state");
                        match state.status() {
                            InvoicePaymentState::Initiaited => {
                                info!("Initiated");
                            }
                            InvoicePaymentState::InFlight => {
                                info!("InFlight");
                            }
                            InvoicePaymentState::Succeeded => {
                                client.settle_htlc(state.preimage()).await.unwrap();
                                break;
                            }
                            InvoicePaymentState::Failed => {
                                error!("Failed");
                            }
                        }
                    }
                    others => {
                        info!("{:?}", others);
                    }
                }
            }
        });
        let correct_state_c = correct_state.clone();
        loop {
            info!("Waiting for state");
            match states.next().await {
                Some(LndWebsocketMessage::Response(state)) => match state.state() {
                    HodlState::OPEN => {
                        info!("Open");
                    }
                    HodlState::ACCEPTED => {
                        lnd_ws.sender.send(pr.clone()).await.unwrap();
                        info!("Sent payment");
                    }
                    HodlState::SETTLED => {
                        info!("REALLY Settled");
                        *correct_state_c.lock().await = true;
                        break;
                    }
                    _ => {}
                },
                _ => {}
            }
        }
        assert!(*correct_state.lock().await);
        Ok(())
    }
}
