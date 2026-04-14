use crate::error::FinanceError;
use futures_util::{SinkExt, StreamExt};
use pyo3::prelude::*;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::protocol::Message;

/// Generic WebSocket client for market data streaming.
#[pyclass]
pub struct MarketStream {
    url: String,
}

#[pymethods]
impl MarketStream {
    #[new]
    pub fn new(url: String) -> PyResult<Self> {
        let parsed_url = url::Url::parse(&url).map_err(FinanceError::UrlParse).map_err(PyErr::from)?;

        let scheme = parsed_url.scheme();
        if scheme != "ws" && scheme != "wss" {
            return Err(PyErr::from(FinanceError::Runtime(
                "Only ws and wss schemes are allowed".to_string(),
            )));
        }

        let host = parsed_url.host_str().ok_or_else(|| {
            PyErr::from(FinanceError::Runtime("URL has no host".to_string()))
        })?;

        if !host.ends_with(".nseindia.com") && host != "nseindia.com" && !host.ends_with(".mcxindia.com") && host != "mcxindia.com" {
            return Err(PyErr::from(FinanceError::Runtime(
                "URL host must be a trusted domain".to_string(),
            )));
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
            crate::runtime().block_on(async {
                let (mut ws_stream, _) = connect_async(&self.url)
                    .await
                    .map_err(|e| FinanceError::Runtime(e.to_string()))?;

                if let Some(msg) = subscribe_msg {
                    ws_stream
                        .send(Message::Text(msg.into()))
                        .await
                        .map_err(|e| FinanceError::Runtime(e.to_string()))?;
                }

                while let Some(msg) = ws_stream.next().await {
                    let msg = msg.map_err(|e| FinanceError::Runtime(e.to_string()))?;

                    if msg.is_text() {
                        let text = msg.to_text().unwrap_or_default();
                        Python::with_gil(|py| {
                            // Prefer a structured Python dict/list when the frame is JSON.
                            let py_val =
                                if let Ok(val) = serde_json::from_str::<serde_json::Value>(text) {
                                    crate::to_py_obj(py, val).unwrap_or_else(|_| {
                                        pyo3::IntoPyObjectExt::into_py_any(text, py)
                                            .expect("&str into Python must not fail")
                                    })
                                } else {
                                    pyo3::IntoPyObjectExt::into_py_any(text, py)
                                        .expect("&str into Python must not fail")
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
            .map_err(PyErr::from)
        })
    }
}
