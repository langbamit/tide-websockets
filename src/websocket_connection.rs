use std::pin::Pin;

use async_dup::{Arc, Mutex};
use async_std::task;
use async_tungstenite::WebSocketStream;
use futures_util::stream::{SplitSink, SplitStream, Stream};
use futures_util::{SinkExt, StreamExt};

use crate::Message;
use tide::http::upgrade::Connection;

/// This is the type that the handler passed to [`WebSocket::new`]
/// receives. It represents a bidirectional stream of websocket data.
#[derive(Clone, Debug)]
pub struct WebSocketConnection(
    Arc<Mutex<SplitSink<WebSocketStream<Connection>, Message>>>,
    Arc<Mutex<SplitStream<WebSocketStream<Connection>>>>,
);

impl WebSocketConnection {
    /// Sends a string message to the connected websocket client
    pub async fn send_string(&self, s: String) -> tide::Result<()> {
        self.0.lock().send(Message::Text(s)).await?;
        Ok(())
    }

    /// Sends a binary message to the connected websocket client
    pub async fn send_bytes(&self, bytes: Vec<u8>) -> tide::Result<()> {
        self.0.lock().send(Message::Binary(bytes)).await?;
        Ok(())
    }

    /// Sends the serde_json serialization of the provided type as a string to the connected websocket client
    pub async fn send_json(&self, json: &impl serde::Serialize) -> tide::Result<()> {
        self.send_string(serde_json::to_string(json)?).await
    }

    pub(crate) fn new(ws: WebSocketStream<Connection>) -> Self {
        let (s, r) = ws.split();
        Self(Arc::new(Mutex::new(s)), Arc::new(Mutex::new(r)))
    }
}

impl Stream for WebSocketConnection {
    type Item = Result<Message, async_tungstenite::tungstenite::Error>;

    fn poll_next(
        self: Pin<&mut Self>,
        cx: &mut task::Context<'_>,
    ) -> task::Poll<Option<Self::Item>> {
        Pin::new(&mut *self.1.lock()).poll_next(cx)
    }
}

impl From<WebSocketStream<Connection>> for WebSocketConnection {
    fn from(ws: WebSocketStream<Connection>) -> Self {
        Self::new(ws)
    }
}
