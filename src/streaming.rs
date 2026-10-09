use crate::error::FinanceError;
use futures_util::{SinkExt, StreamExt};
use pyo3::prelude::*;
use std::time::Duration;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::time::{Instant, sleep_until, timeout};
use tokio_tungstenite::tungstenite::protocol::Message;
use tokio_tungstenite::{WebSocketStream, connect_async};

const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(30);
const PONG_TIMEOUT: Duration = Duration::from_secs(10);

struct Heartbeat {
    deadline: Instant,
    pending: Option<bytes::Bytes>,
    sequence: u64,
}

impl Heartbeat {
    fn new() -> Self {
        Self {
            deadline: Instant::now() + HEARTBEAT_INTERVAL,
            pending: None,
            sequence: 0,
        }
    }

    async fn next_message<S>(
        &mut self,
        stream: &mut WebSocketStream<S>,
    ) -> Result<Option<Message>, FinanceError>
    where
        S: AsyncRead + AsyncWrite + Unpin,
    {
        loop {
            tokio::select! {
                // Check timers first so a busy stream cannot starve the watchdog.
                biased;
                _ = sleep_until(self.deadline) => {
                    if self.pending.is_some() {
                        return Err(FinanceError::Runtime("WebSocket heartbeat Pong timed out".into()));
                    }
                    self.sequence = self.sequence.wrapping_add(1);
                    let payload = bytes::Bytes::copy_from_slice(&self.sequence.to_be_bytes());
                    timeout(
                        crate::common::DEFAULT_CONNECT_TIMEOUT,
                        stream.send(Message::Ping(payload.clone())),
                    )
                    .await
                    .map_err(|_| FinanceError::Runtime("WebSocket heartbeat Ping send timed out".into()))?
                    .map_err(|e| FinanceError::Runtime(e.to_string()))?;
                    self.pending = Some(payload);
                    self.deadline = Instant::now() + PONG_TIMEOUT;
                }
                message = stream.next() => {
                    let Some(message) = message else { return Ok(None) };
                    let message = message.map_err(|e| FinanceError::Runtime(e.to_string()))?;
                    if let Message::Pong(payload) = &message {
                        if self.pending.as_ref() == Some(payload) {
                            self.pending = None;
                            self.deadline = Instant::now() + HEARTBEAT_INTERVAL;
                        }
                    } else {
                        // Application traffic never extends the heartbeat deadline.
                        return Ok(Some(message));
                    }
                }
            }
        }
    }
}

/// Generic WebSocket client for market data streaming.
#[pyclass]
pub struct MarketStream {
    url: String,
}

#[pymethods]
impl MarketStream {
    #[new]
    pub fn new(url: String) -> PyResult<Self> {
        let parsed_url = url::Url::parse(&url)
            .map_err(|e| pyo3::exceptions::PyValueError::new_err(format!("Invalid URL: {}", e)))?;

        let scheme = parsed_url.scheme();
        let is_test = std::env::var("FINANCEINDIA_TEST_ENV").as_deref() == Ok("1");

        if is_test {
            if scheme != "ws" && scheme != "wss" {
                return Err(pyo3::exceptions::PyValueError::new_err(
                    "Invalid URL scheme. Only 'ws' and 'wss' are allowed for streaming.",
                ));
            }
        } else {
            if scheme != "wss" {
                return Err(pyo3::exceptions::PyValueError::new_err(
                    "Invalid URL scheme. Only 'wss' (secure) is allowed for streaming.",
                ));
            }
        }

        let host = parsed_url
            .host_str()
            .ok_or_else(|| pyo3::exceptions::PyValueError::new_err("URL has no host"))?;

        if !(host.ends_with(".nseindia.com") || host == "nseindia.com")
            && !(host.ends_with(".mcxindia.com") || host == "mcxindia.com")
        {
            return Err(pyo3::exceptions::PyValueError::new_err(
                "Invalid domain: only nseindia.com, mcxindia.com and their subdomains are allowed.",
            ));
        }

        Ok(MarketStream { url })
    }

    /// Starts listening to the market stream.
    ///
    /// Blocks the calling thread and invokes `callback(message)` for each incoming frame.
    /// `message` is a parsed Python object when the frame contains valid JSON, otherwise
    /// a raw `str` for text frames or `bytes` for binary frames.
    ///
    /// `subscribe_msg`: optional JSON string sent to the server immediately after connecting.
    /// A Ping is sent after each 30-second heartbeat interval; its matching Pong
    /// must arrive within 10 seconds, even if application frames are arriving.
    ///
    /// # Stopping the stream
    /// Raise an exception inside the callback to abort the loop.
    fn listen(
        &self,
        py: Python<'_>,
        callback: PyObject,
        subscribe_msg: Option<String>,
    ) -> PyResult<()> {
        py.allow_threads(|| {
            crate::runtime()
                .block_on(async {
                    let connect_future = connect_async(&self.url);
                    let (mut ws_stream, _) = tokio::time::timeout(
                        crate::common::DEFAULT_CONNECT_TIMEOUT,
                        connect_future,
                    )
                    .await
                    .map_err(|_| {
                        FinanceError::Runtime("WebSocket connection timed out".to_string())
                    })?
                    .map_err(|e| FinanceError::Runtime(e.to_string()))?;

                    if let Some(msg) = subscribe_msg {
                        tokio::time::timeout(
                            crate::common::DEFAULT_CONNECT_TIMEOUT,
                            ws_stream.send(Message::Text(msg.into())),
                        )
                        .await
                        .map_err(|_| FinanceError::Runtime("WebSocket send timed out".to_string()))?
                        .map_err(|e| FinanceError::Runtime(e.to_string()))?;
                    }

                    let mut heartbeat = Heartbeat::new();
                    while let Some(msg) = heartbeat.next_message(&mut ws_stream).await? {
                        if msg.is_text() {
                            let text = msg.to_text().unwrap_or_default();
                            Python::with_gil(|py| -> PyResult<_> {
                                let py_val = if let Ok(val) =
                                    serde_json::from_str::<serde_json::Value>(text)
                                {
                                    match crate::to_py_obj(py, val) {
                                        Ok(obj) => obj,
                                        Err(_) => pyo3::IntoPyObjectExt::into_py_any(text, py)?,
                                    }
                                } else {
                                    pyo3::IntoPyObjectExt::into_py_any(text, py)?
                                };
                                callback.call1(py, (py_val,))
                            })
                            .map_err(FinanceError::from)?;
                        } else if msg.is_binary() {
                            let data = msg.into_data();
                            Python::with_gil(|py| {
                                let py_data = pyo3::types::PyBytes::new(py, &data);
                                callback.call1(py, (py_data,))
                            })
                            .map_err(FinanceError::from)?;
                        }
                    }
                    Ok::<(), FinanceError>(())
                })
                .map_err(|e| Python::with_gil(|_| PyErr::from(e)))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::{DuplexStream, duplex};
    use tokio_tungstenite::tungstenite::protocol::Role;

    async fn pair(
        capacity: usize,
    ) -> (WebSocketStream<DuplexStream>, WebSocketStream<DuplexStream>) {
        let (client, server) = duplex(capacity);
        (
            WebSocketStream::from_raw_socket(client, Role::Client, None).await,
            WebSocketStream::from_raw_socket(server, Role::Server, None).await,
        )
    }

    #[tokio::test(start_paused = true)]
    async fn quiet_stream_survives_multiple_heartbeats() {
        let (mut client, mut server) = pair(1024).await;
        let mut heartbeat = Heartbeat::new();
        let started = Instant::now();
        let peer = async {
            let mut previous_payload = None;
            for _ in 0..3 {
                let Message::Ping(payload) = server.next().await.unwrap().unwrap() else {
                    panic!("expected heartbeat Ping");
                };
                assert_ne!(previous_payload.as_ref(), Some(&payload));
                previous_payload = Some(payload);
                // Tungstenite queues the matching Pong when it reads the Ping.
                server.flush().await.unwrap();
            }
            server
                .send(Message::Text("quiet but alive".into()))
                .await
                .unwrap();
        };
        let (message, ()) = tokio::join!(heartbeat.next_message(&mut client), peer);
        assert_eq!(
            message.unwrap(),
            Some(Message::Text("quiet but alive".into()))
        );
        assert!(started.elapsed() >= HEARTBEAT_INTERVAL * 3);
    }

    #[tokio::test(start_paused = true)]
    async fn silent_peer_times_out_waiting_for_pong() {
        let (mut client, _server) = pair(1024).await;
        let mut heartbeat = Heartbeat::new();
        let started = Instant::now();
        let error = heartbeat.next_message(&mut client).await.unwrap_err();
        assert!(error.to_string().contains("heartbeat Pong timed out"));
        assert_eq!(started.elapsed(), HEARTBEAT_INTERVAL + PONG_TIMEOUT);
    }

    #[tokio::test(start_paused = true)]
    async fn stale_pongs_and_application_frames_do_not_extend_deadline() {
        let (mut client, mut server) = pair(4096).await;
        let mut heartbeat = Heartbeat::new();
        let started = Instant::now();
        let listener = async {
            let mut received = 0;
            loop {
                match heartbeat.next_message(&mut client).await {
                    Ok(Some(Message::Text(_))) | Ok(Some(Message::Binary(_))) => received += 1,
                    Err(error) => {
                        assert!(error.to_string().contains("heartbeat Pong timed out"));
                        assert!(received > 0);
                        break;
                    }
                    other => panic!("unexpected result: {other:?}"),
                }
            }
        };
        let peer = async {
            let Message::Ping(first) = server.next().await.unwrap().unwrap() else {
                panic!("expected first Ping");
            };
            server.flush().await.unwrap();
            let Message::Ping(second) = server.next().await.unwrap().unwrap() else {
                panic!("expected second Ping");
            };
            assert_ne!(first, second);
            // An explicit Pong replaces tungstenite's queued automatic Pong.
            server.send(Message::Pong(first.clone())).await.unwrap();
            for _ in 0..9 {
                server.send(Message::Text("tick".into())).await.unwrap();
                server
                    .send(Message::Binary(vec![1, 2, 3].into()))
                    .await
                    .unwrap();
                server.send(Message::Pong(first.clone())).await.unwrap();
                tokio::time::sleep(Duration::from_secs(1)).await;
            }
        };
        tokio::join!(listener, peer);
        assert_eq!(started.elapsed(), HEARTBEAT_INTERVAL * 2 + PONG_TIMEOUT);
    }

    #[tokio::test(start_paused = true)]
    async fn blocked_ping_write_times_out() {
        let (mut client, _server) = pair(1).await;
        let mut heartbeat = Heartbeat::new();
        let started = Instant::now();
        let error = heartbeat.next_message(&mut client).await.unwrap_err();
        assert!(error.to_string().contains("heartbeat Ping send timed out"));
        assert_eq!(
            started.elapsed(),
            HEARTBEAT_INTERVAL + crate::common::DEFAULT_CONNECT_TIMEOUT
        );
    }

    #[tokio::test(start_paused = true)]
    async fn close_and_transport_errors_are_preserved() {
        let (mut client, mut server) = pair(1024).await;
        server.send(Message::Close(None)).await.unwrap();
        assert_eq!(
            Heartbeat::new().next_message(&mut client).await.unwrap(),
            Some(Message::Close(None))
        );

        let (mut client, server) = pair(1024).await;
        drop(server);
        let error = Heartbeat::new()
            .next_message(&mut client)
            .await
            .unwrap_err();
        assert!(!error.to_string().contains("timed out"));
    }
}
