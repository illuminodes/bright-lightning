use std::sync::Arc;

use futures_util::{
    stream::{SplitSink, SplitStream},
    SinkExt, StreamExt,
};
use httparse::Header;
use serde::{de::DeserializeOwned, Serialize};
use tokio::{net::TcpStream, sync::RwLock};
use tokio_tungstenite::{tungstenite::Message, MaybeTlsStream, WebSocketStream};

use crate::{LndError, LndResponse};

type LndWebsocketWriterHalf =
    Option<SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>>;
type LndWebsocketReaderHalf = Option<SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>>;

#[derive(Clone)]
pub struct LndWebsocketWriter(Arc<RwLock<LndWebsocketWriterHalf>>);
impl LndWebsocketWriter {
    pub fn new(writer: LndWebsocketWriterHalf) -> Self {
        Self(Arc::new(RwLock::new(writer)))
    }
    pub async fn send<S>(&self, message: S) -> anyhow::Result<()>
    where
        S: TryInto<String> + Send + Sync + 'static,
        <S as TryInto<std::string::String>>::Error:
            std::marker::Send + std::fmt::Debug + std::marker::Sync,
    {
        let message_string = message
            .try_into()
            .map_err(|_e| anyhow::anyhow!("Could not parse"))?;
        let message = Message::Text(message_string.into());
        if let Some(writer) = self.0.write().await.as_mut() {
            writer.send(message).await.map_err(|e| e.into())
        } else {
            Err(anyhow::anyhow!("No writer"))
        }
    }
}
#[derive(Clone)]
pub struct LndWebsocketReader(Arc<RwLock<LndWebsocketReaderHalf>>);
impl LndWebsocketReader {
    pub fn new(reader: LndWebsocketReaderHalf) -> Self {
        Self(Arc::new(RwLock::new(reader)))
    }
    pub async fn read<R>(&self) -> Option<LndWebsocketMessage<R>>
    where
        R: TryFrom<String>
            + TryInto<String>
            + Send
            + Sync
            + 'static
            + Serialize
            + DeserializeOwned
            + Clone,
        <R as TryFrom<std::string::String>>::Error: std::marker::Send + std::fmt::Debug,
    {
        let mut reader = self.0.write().await;
        let message = reader.as_mut()?.next().await?;
        match message {
            Ok(message) => match message {
                Message::Text(text) => match LndResponse::<R>::try_from(&text.to_string()) {
                    Ok(response) => Some(LndWebsocketMessage::Response(response.inner())),
                    Err(_e) => {
                        let lnd_error = LndError::try_from(text.to_string()).ok()?;
                        Some(LndWebsocketMessage::Error(lnd_error))
                    }
                },
                Message::Ping(_) => Some(LndWebsocketMessage::Ping),
                _ => None,
            },
            Err(e) => Some(LndWebsocketMessage::Error(
                LndError::try_from(e.to_string()).unwrap(),
            )),
        }
    }
}

#[derive(Debug)]
pub enum LndWebsocketMessage<R> {
    Response(R),
    Error(LndError),
    Ping,
}
pub struct LndWebsocket {
    pub receiver: LndWebsocketReader,
    pub sender: LndWebsocketWriter,
}

impl LndWebsocket {
    pub fn new() -> Self {
        Self {
            receiver: LndWebsocketReader::new(None),
            sender: LndWebsocketWriter::new(None),
        }
    }
    pub async fn connect(
        &self,
        url: String,
        macaroon: String,
        request: String,
    ) -> anyhow::Result<Self> {
        let random_key = "dGhlIHNhbXBsZSBub25jZQ2342qdfsdgfsdfg";
        let mut headers = [
            Header {
                name: "Grpc-Metadata-macaroon",
                value: macaroon.as_bytes(),
            },
            Header {
                name: "Sec-WebSocket-Key",
                value: random_key.as_bytes(),
            },
            Header {
                name: "Host",
                value: url.as_bytes(),
            },
            Header {
                name: "Connection",
                value: "Upgrade".as_bytes(),
            },
            Header {
                name: "Upgrade",
                value: "websocket".as_bytes(),
            },
            httparse::Header {
                name: "Sec-WebSocket-Version",
                value: "13".as_bytes(),
            },
        ];
        let mut req = httparse::Request::new(&mut headers);
        req.method = Some("GET");
        req.path = Some(&request);
        req.version = Some(1);

        // Prepare the websocket connection with SSL
        let danger_conf = Some(
            tokio_tungstenite::tungstenite::protocol::WebSocketConfig::default()
                .accept_unmasked_frames(true),
        );

        let tls = native_tls::TlsConnector::builder()
            .danger_accept_invalid_certs(true)
            .build()?;
        let (ws, _response) = tokio_tungstenite::connect_async_tls_with_config(
            req,
            danger_conf,
            false,
            Some(tokio_tungstenite::Connector::NativeTls(tls)),
        )
        .await?;
        let (websocket_sender, websocket_reader) = ws.split();
        let sender = LndWebsocketWriter::new(Some(websocket_sender));
        let receiver = LndWebsocketReader::new(Some(websocket_reader));
        Ok(Self { receiver, sender })
    }
}
#[cfg(test)]
mod test {

    use super::LndWebsocketMessage;
    use crate::LndHodlInvoiceState;
    use std::io::Read;
    use tracing_test::traced_test;

    #[tokio::test]
    #[traced_test]
    async fn check_invoice_paid() -> Result<(), anyhow::Error> {
        let url = "lnd.illuminodes.com";
        let client = crate::lnd::rest_client::LightningClient::new(url, "./admin.macaroon").await?;
        let invoice = client
            .get_invoice(crate::LndInvoiceRequestBody {
                value: 1000.to_string(),
                memo: Some("Hello".to_string()),
            })
            .await?;
        tracing::info!("Invoice: {}", invoice);
        let query = format!(
            "wss://{}/v2/invoices/subscribe/{}",
            url,
            invoice.r_hash_url_safe()
        );
        let mut macaroon = vec![];
        let mut file = std::fs::File::open("./admin.macaroon")?;
        file.read_to_end(&mut macaroon)?;
        let lnd_ws = super::LndWebsocket::new()
            .connect(
                url.to_string(),
                macaroon.iter().map(|b| format!("{:02x}", b)).collect(),
                query,
            )
            .await?;
        loop {
            match lnd_ws.receiver.read::<LndHodlInvoiceState>().await {
                Some(LndWebsocketMessage::Response(state)) => {
                    tracing::info!("State: {}", state);
                    break;
                }
                Some(LndWebsocketMessage::Error(e)) => {
                    tracing::error!("Error: {}", e);
                    assert!(false);
                }
                Some(LndWebsocketMessage::Ping) => {
                    tracing::info!("Ping");
                }
                None => {
                    tracing::info!("None");
                    assert!(false);
                }
            }
        }
        Ok(())
    }
}
