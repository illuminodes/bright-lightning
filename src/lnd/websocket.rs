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

#[derive(Clone, Default, Debug)]
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
        let mut writer = self.0.write().await;
        if let Some(writer) = writer.as_mut() {
            Ok(writer.send(message).await?)
        } else {
            Err(anyhow::anyhow!("No writer"))
        }
    }
}
#[derive(Clone, Default, Debug)]
pub struct LndWebsocketReader(Arc<RwLock<LndWebsocketReaderHalf>>);
impl LndWebsocketReader {
    #[must_use]
    pub fn new(reader: LndWebsocketReaderHalf) -> Self {
        Self(Arc::new(RwLock::new(reader)))
    }
    pub async fn read<R>(&self) -> Option<LndWebsocketMessage<R>>
    where
        R: TryFrom<String>
            + std::fmt::Display
            + Send
            + Sync
            + 'static
            + Serialize
            + DeserializeOwned
            + Clone,
        <R as TryFrom<std::string::String>>::Error: std::marker::Send + std::fmt::Debug,
    {
        let value = self.0.write().await.as_mut()?.next().await?;
        match value {
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

#[derive(Debug, Default)]
pub struct LndWebsocket {
    pub receiver: LndWebsocketReader,
    pub sender: LndWebsocketWriter,
}

impl LndWebsocket {
    pub async fn connect(
        &self,
        url: String,
        macaroon: String,
        request: String,
    ) -> anyhow::Result<Self> {
        let random_key = b"dGhlIHNhbXBsZSBub25jZQ2342qdfsdgfsdfg";
        let mut headers = [
            Header {
                name: "Grpc-Metadata-macaroon",
                value: macaroon.as_bytes(),
            },
            Header {
                name: "Sec-WebSocket-Key",
                value: random_key,
            },
            Header {
                name: "Host",
                value: url.as_bytes(),
            },
            Header {
                name: "Connection",
                value: b"Upgrade",
            },
            Header {
                name: "Upgrade",
                value: b"websocket",
            },
            httparse::Header {
                name: "Sec-WebSocket-Version",
                value: b"13",
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
        let client = crate::lnd::rest_client::LndRestClient::new(url, "./admin.macaroon")?;
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
        let lnd_ws = super::LndWebsocket::default()
            .connect(
                url.to_string(),
                macaroon.iter().fold(String::new(), |mut acc, x| {
                    acc.push_str(&format!("{x:02x}"));
                    acc
                }),
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
