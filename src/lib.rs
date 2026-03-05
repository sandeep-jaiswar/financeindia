use pyo3::prelude::*;
use pyo3::exceptions::PyRuntimeError;
use reqwest::blocking::{Client, ClientBuilder};
use reqwest::header::{HeaderMap, HeaderValue, USER_AGENT, ACCEPT, ACCEPT_LANGUAGE, REFERER, CACHE_CONTROL, PRAGMA};
use std::time::Duration;

#[pyclass]
struct FinanceClient {
    client: Client,
}

#[pymethods]
impl FinanceClient {
    #[new]
    fn new() -> PyResult<Self> {
        let mut headers = HeaderMap::new();
        
        // Exact headers from your working example
        headers.insert(USER_AGENT, HeaderValue::from_static("Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/121.0.0.0 Safari/537.36"));
        headers.insert(ACCEPT, HeaderValue::from_static("*/*"));
        headers.insert(ACCEPT_LANGUAGE, HeaderValue::from_static("en-US,en;q=0.9"));
        headers.insert(CACHE_CONTROL, HeaderValue::from_static("no-cache"));
        headers.insert(PRAGMA, HeaderValue::from_static("no-cache"));

        let client = ClientBuilder::new()
            .default_headers(headers)
            .cookie_store(true)
            .timeout(Duration::from_secs(1))
            .build()
            .map_err(|e| PyErr::new::<PyRuntimeError, _>(e.to_string()))?;

        Ok(FinanceClient { client })
    }

    fn _initialize_session(&self, py: Python<'_>) -> PyResult<()> {
        py.allow_threads(|| {
            // Must hit the home page first to "bake" the cookies in the Jar
            let response = self.client.get("https://www.nseindia.com/")
                .send()
                .map_err(|e| PyErr::new::<PyRuntimeError, _>(e.to_string()))?;
            
            response.error_for_status()
                .map_err(|e| PyErr::new::<PyRuntimeError, _>(e.to_string()))?;
            Ok(())
        })
    }

    fn get_market_status(&self, py: Python<'_>) -> PyResult<String> {
        py.allow_threads(|| {
            // 1. Refresh cookies by hitting a landing page (mimic human behavior)
            let _ = self.client.get("https://www.nseindia.com/all-reports").send();

            // 2. The actual API call
            let response = self.client.get("https://www.nseindia.com/api/marketStatus")
                .header(REFERER, "https://www.nseindia.com/all-reports")
                .send()
                .map_err(|e| PyErr::new::<PyRuntimeError, _>(e.to_string()))?;

            let checked_response = response.error_for_status()
                .map_err(|e| PyErr::new::<PyRuntimeError, _>(e.to_string()))?;

            checked_response.text()
                .map_err(|e| PyErr::new::<PyRuntimeError, _>(e.to_string()))
        })
    }
}

#[pymodule]
fn financeindia(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<FinanceClient>()?;
    Ok(())
}