use crate::{
    emit,
    event::Event,
    internal_events::{SocketMode, SocketReceiveError, UnixSocketFileDeleteFailed},
    shutdown::ShutdownSignal,
    sources::Source,
    Pipeline,
};
use bytes::{Bytes, BytesMut};
use futures::SinkExt;
use std::{borrow::Borrow, fs::remove_file, path::PathBuf};
use tokio::net::UnixDatagram;
use tokio_util::codec::Decoder;
use tracing::field;

/// Returns a Source object corresponding to a Unix domain datagram
/// socket.  Passing in different functions for build_event can allow
/// for different source-specific logic (such as decoding syslog
/// messages in the syslog source).
pub fn build_unix_datagram_source(
    listen_path: PathBuf,
    max_length: usize,
    host_key: String,
    shutdown: ShutdownSignal,
    out: Pipeline,
    build_event: impl Fn(&str, Option<&str>, &[u8]) -> Option<Event> + Clone + Send + Sync + 'static,
) -> Source {
    Box::pin(async move {
        let socket = UnixDatagram::bind(&listen_path).expect("Failed to bind to datagram socket");
        info!(message = "Listening.", path = ?listen_path, r#type = "unix_datagram");

        let result = listen(socket, max_length, host_key, shutdown, out, build_event).await;

        // Delete socket file
        if let Err(error) = remove_file(&listen_path) {
            emit!(UnixSocketFileDeleteFailed {
                path: &listen_path,
                error
            });
        }

        result
    })
}

async fn listen(
    socket: UnixDatagram,
    max_length: usize,
    host_key: String,
    mut shutdown: ShutdownSignal,
    out: Pipeline,
    build_event: impl Fn(&str, Option<&str>, &[u8]) -> Option<Event> + Clone + Send + Sync + 'static,
) -> Result<(), ()> {
    let mut out = out.sink_map_err(|error| error!(message = "Error sending line.", %error));
    let mut buf = BytesMut::with_capacity(max_length);
    loop {
        buf.resize(max_length, 0);
        tokio::select! {
            recv = socket.recv_from(&mut buf) => {
                let (byte_size, address) = recv.map_err(|error| {
                    emit!(SocketReceiveError { error, mode: SocketMode::Unix })
                })?;

                let mut payload = buf.split_to(byte_size);

                let span = info_span!("datagram");
                let path = address.as_pathname().map(|e| e.to_owned()).map(|path| {
                    span.record("peer_path", &field::debug(&path));
                    path
                });

                let received_from: Option<String> =
                    path.map(|p| p.to_string_lossy().to_string());
                let received_from: Option<&str> = received_from.map(|path| path.as_str());

                if let Some(event) = build_event(&host_key, received_from.clone(), &payload) {
                    out.send(event).await?;
                }
            }
            _ = &mut shutdown => return Ok(()),
        }
    }
}
