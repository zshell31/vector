use crate::{
    buffers::Acker,
    config::{DataType, GenerateConfig, SinkConfig, SinkContext, SinkDescription},
    dns, emit,
    event::Event,
    internal_events::{WebSocketEventSendFail, WebSocketEventSendSuccess},
    sinks::util::{
        encoding::{EncodingConfig, EncodingConfiguration},
        StreamSink,
    },
    tls::{MaybeTlsSettings, MaybeTlsStream, TlsConfig, TlsError},
};
use async_trait::async_trait;
use futures::{sink::SinkExt, stream::BoxStream, Sink, StreamExt};
use serde::{Deserialize, Serialize};
use snafu::{ResultExt, Snafu};
use std::{error::Error, fmt::Debug, net::SocketAddr};
use tokio::net::TcpStream;
use tokio_tungstenite::{
    client_async_with_config,
    tungstenite::{
        client::{uri_mode, IntoClientRequest},
        error::{Error as WsError, UrlError},
        handshake::client::Request as WsRequest,
        protocol::{Message, WebSocketConfig},
        stream::Mode as UriMode,
    },
    WebSocketStream as WsStream,
};

#[derive(Debug, Snafu)]
enum WebSocketError {
    #[snafu(display("Creating WebSocket client failed: {}", source))]
    CreateFailed { source: WsError },
    #[snafu(display("Connect error: {}", source))]
    ConnectError { source: TlsError },
    #[snafu(display("Unable to resolve DNS: {}", source))]
    DnsError { source: dns::DnsError },
    #[snafu(display("No addresses returned."))]
    NoAddresses,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct WebSocketSinkConfig {
    uri: String,
    tls: Option<TlsConfig>,
    encoding: EncodingConfig<Encoding>,
}

#[derive(Clone, Copy, Debug, Derivative, Deserialize, Serialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Encoding {
    Text,
    Json,
}

inventory::submit! {
    SinkDescription::new::<WebSocketSinkConfig>("websocket")
}

impl GenerateConfig for WebSocketSinkConfig {
    fn generate_config() -> toml::Value {
        toml::from_str(
            r#"uri = "ws://127.0.0.1:9000/endpoint"
            encoding.codec = "json""#,
        )
        .unwrap()
    }
}

#[async_trait::async_trait]
#[typetag::serde(name = "websocket")]
impl SinkConfig for WebSocketSinkConfig {
    async fn build(
        &self,
        cx: SinkContext,
    ) -> crate::Result<(super::VectorSink, super::Healthcheck)> {
        let connector = self.build_connector()?;
        let ws_stream = self.build_ws_stream(&connector).await?;
        let ws_sink = WebSocketSink::new(self.clone(), ws_stream, cx.acker());

        Ok((
            super::VectorSink::Stream(Box::new(ws_sink)),
            Box::pin(async move { connector.healthcheck().await }),
        ))
    }

    fn input_type(&self) -> DataType {
        DataType::Log
    }

    fn sink_type(&self) -> &'static str {
        "websocket"
    }
}

#[derive(Clone)]
struct WebSocketConnector {
    host: String,
    port: u16,
    tls: MaybeTlsSettings,
}

impl WebSocketSinkConfig {
    fn build_connector(&self) -> Result<WebSocketConnector, WebSocketError> {
        let request = (&self.uri).into_client_request().context(CreateFailed)?;
        let (host, port) = Self::extract_host_and_port(&request).context(CreateFailed)?;
        let tls = MaybeTlsSettings::from_config(&self.tls, false).context(ConnectError)?;

        Ok(WebSocketConnector::new(host, port, tls))
    }

    async fn build_ws_stream(
        &self,
        connector: &WebSocketConnector,
    ) -> Result<WsStream<MaybeTlsStream<TcpStream>>, WebSocketError> {
        let request = (&self.uri).into_client_request().context(CreateFailed)?;
        let maybe_tls_stream = connector.connect().await?;

        let ws_config = WebSocketConfig {
            max_send_queue: None, // don't buffer messages
            ..Default::default()
        };

        let (ws_stream, _response) =
            client_async_with_config(request, maybe_tls_stream, Some(ws_config))
                .await
                .context(CreateFailed)?;

        Ok(ws_stream)
    }

    fn extract_host_and_port(request: &WsRequest) -> Result<(String, u16), WsError> {
        let host = request
            .uri()
            .host()
            .ok_or(WsError::Url(UrlError::NoHostName))?
            .to_string();
        let mode = uri_mode(&request.uri())?;
        let port = request.uri().port_u16().unwrap_or_else(|| match mode {
            UriMode::Tls => 443,
            UriMode::Plain => 80,
        });

        Ok((host, port))
    }
}

impl WebSocketConnector {
    fn new(host: String, port: u16, tls: MaybeTlsSettings) -> Self {
        Self { host, port, tls }
    }

    async fn connect(&self) -> Result<MaybeTlsStream<TcpStream>, WebSocketError> {
        let ip = dns::Resolver
            .lookup_ip(self.host.clone())
            .await
            .context(DnsError)?
            .next()
            .ok_or(WebSocketError::NoAddresses)?;

        let addr = SocketAddr::new(ip, self.port);
        self.tls
            .connect(&self.host, &addr)
            .await
            .context(ConnectError)
    }

    async fn healthcheck(&self) -> crate::Result<()> {
        self.connect().await.map(|_| ()).map_err(Into::into)
    }
}

pub struct WebSocketSink<WS> {
    encoding: EncodingConfig<Encoding>,
    ws_stream: WS,
    acker: Acker,
}

impl<WS> WebSocketSink<WS> {
    fn new(config: WebSocketSinkConfig, ws_stream: WS, acker: Acker) -> Self {
        Self {
            encoding: config.encoding,
            ws_stream,
            acker,
        }
    }
}

#[async_trait]
impl<WS> StreamSink for WebSocketSink<WS>
where
    WS: Sink<Message> + Unpin + Send,
    <WS as Sink<Message>>::Error: Error + Debug,
{
    async fn run(&mut self, mut input: BoxStream<'_, Event>) -> Result<(), ()> {
        while let Some(event) = input.next().await {
            let log = encode_event(event, &self.encoding);

            if let Some(msg) = log {
                let msg_len = msg.len();
                match self.ws_stream.send(msg).await {
                    Ok(_) => emit!(WebSocketEventSendSuccess { byte_size: msg_len }),
                    Err(error) => emit!(WebSocketEventSendFail { error }),
                }
            }

            self.acker.ack(1);
        }

        Ok(())
    }
}

fn encode_event(mut event: Event, encoding: &EncodingConfig<Encoding>) -> Option<Message> {
    encoding.apply_rules(&mut event);

    let msg = match encoding.codec() {
        Encoding::Json => serde_json::to_string(event.as_log())
            .map_err(|error| error!(message = "Unable to encode.", %error))
            .ok(),
        Encoding::Text => event
            .as_log()
            .get(crate::config::log_schema().message_key())
            .map(|v| v.to_string_lossy()),
    };

    msg.map(|msg| Message::text(msg))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        config::SinkContext,
        event::{Event, Value as EventValue},
        test_util::{next_addr, random_lines_with_stream, trace_init, CountReceiver},
        tls::{self, TlsOptions},
    };
    use futures::{future::ready, stream::StreamExt};
    use serde_json::Value as JsonValue;
    use std::net::SocketAddr;
    use tokio_tungstenite::{
        accept_async,
        tungstenite::{
            error::{Error as WsError, ProtocolError},
            Message,
        },
    };

    #[test]
    fn generate_config() {
        crate::test_util::test_generate_config::<WebSocketSinkConfig>();
    }

    #[test]
    fn encodes_raw_logs() {
        let event = Event::from("foo");
        assert_eq!(
            Message::text("foo"),
            encode_event(event, &EncodingConfig::from(Encoding::Text)).unwrap()
        );
    }

    #[test]
    fn encodes_log_events() {
        let mut event = Event::new_empty_log();

        let log = event.as_mut_log();
        log.insert("str", EventValue::from("bar"));
        log.insert("num", EventValue::from(10));

        let encoded = encode_event(event, &EncodingConfig::from(Encoding::Json));
        let expected = Message::text(r#"{"num":10,"str":"bar"}"#);
        assert_eq!(expected, encoded.unwrap());
    }

    #[tokio::test]
    async fn test_websocket() {
        trace_init();

        let addr = next_addr();
        let config = WebSocketSinkConfig {
            uri: format!("ws://{}", addr.to_string()),
            tls: None,
            encoding: Encoding::Json.into(),
        };
        let tls = MaybeTlsSettings::Raw(());

        send_events_and_assert(addr, config, tls).await;
    }

    #[cfg(feature = "sources-utils-tls")]
    #[tokio::test]
    async fn test_tls_websocket() {
        trace_init();

        let addr = next_addr();
        let tls_config = Some(TlsConfig::test_config());
        let tls = MaybeTlsSettings::from_config(&tls_config, true).unwrap();

        let config = WebSocketSinkConfig {
            uri: format!("wss://{}", addr.to_string()),
            tls: Some(TlsConfig {
                enabled: Some(true),
                options: TlsOptions {
                    verify_certificate: Some(false),
                    verify_hostname: Some(true),
                    ca_file: Some(tls::TEST_PEM_CRT_PATH.into()),
                    ..Default::default()
                },
            }),
            encoding: Encoding::Json.into(),
        };

        send_events_and_assert(addr, config, tls).await;
    }

    async fn send_events_and_assert(
        addr: SocketAddr,
        config: WebSocketSinkConfig,
        tls: MaybeTlsSettings,
    ) {
        let mut receiver = create_count_receiver(addr, tls);

        let context = SinkContext::new_test();
        let (sink, _healthcheck) = config.build(context).await.unwrap();

        let (lines, events) = random_lines_with_stream(10, 100, None);
        sink.run(events).await.unwrap();

        receiver.connected().await;

        let output = receiver.await;
        assert_eq!(lines.len(), output.len());
        let message_key = crate::config::log_schema().message_key();
        for (source, received) in lines.iter().zip(output) {
            let json = serde_json::from_str::<JsonValue>(&received).expect("Invalid JSON");
            let received = json.get(message_key).unwrap().as_str().unwrap();
            assert_eq!(source, received);
        }
    }

    fn create_count_receiver(addr: SocketAddr, tls: MaybeTlsSettings) -> CountReceiver<String> {
        CountReceiver::receive_items_stream(move |tripwire, connected| async move {
            let listener = tls.bind(&addr).await.unwrap();
            let stream = listener.accept_stream();

            let mut connected = Some(connected);
            stream
                .take_until(tripwire)
                .filter_map(|maybe_tls_stream| async move {
                    let maybe_tls_stream = maybe_tls_stream.unwrap();
                    let ws_stream = accept_async(maybe_tls_stream).await.unwrap();

                    Some(ws_stream.filter_map(|msg| {
                        ready(match msg {
                            Ok(msg) if msg.is_text() => Some(Ok(msg.into_text().unwrap())),
                            Err(WsError::Protocol(ProtocolError::ResetWithoutClosingHandshake)) => {
                                None
                            }
                            Err(e) => Some(Err(e)),
                            _ => None,
                        })
                    }))
                })
                .map(move |ws_stream| {
                    connected.take().map(|trigger| trigger.send(()));
                    ws_stream
                })
                .flatten()
                .map(|msg| msg.unwrap())
        })
    }
}
