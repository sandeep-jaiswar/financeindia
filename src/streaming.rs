use futures_util::{SinkExt, StreamExt};
use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use url::Url;

/// Generic WebSocket client for market data streaming.
#[pyclass]
pub struct MarketStream {
    url: String,
}

#[pymethods]
impl MarketStream {
    #[new]
    pub fn new(url: String) -> PyResult<Self> {
        // Validate URL
        url::Url::parse(&url).map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyValueError, _>(format!("Invalid URL: {}", e))
        })?;
        Ok(MarketStream { url })
    }

    /// Starts listening to the market stream.
    /// Blocks the current thread and calls the provided python callback for each message.
    fn listen(&self, py: Python<'_>, callback: PyObject) -> PyResult<()> {
        py.allow_threads(|| {
            crate::runtime().block_on(async {
                let (mut ws_stream, _) = connect_async(&self.url).await.map_err(|e| {
                    PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string())
                })?;

                while let Some(msg) = ws_stream.next().await {
                    let msg = msg.map_err(|e| {
                        PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string())
                    })?;

                    if msg.is_text() || msg.is_binary() {
                        let data = msg.into_data();
                        Python::with_gil(|py| {
                            let py_data = pyo3::types::PyBytes::new(py, &data);
                            callback.call1(py, (py_data,))
                        })?;
                    }
                }
                Ok::<(), PyErr>(())
            })
        })
    }
}
