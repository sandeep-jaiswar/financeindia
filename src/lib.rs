use pyo3::prelude::*;
use pyo3::exceptions::PyRuntimeError;
use reqwest::blocking::Client;
use std::time::Duration;

#[pyclass]
struct FinanceClient {
    client: Client,
}

#[pymethods]
impl FinanceClient {
    #[new]
    fn new() -> PyResult<Self> {
        let client = Client::builder()
            .cookie_store(true) 
            .timeout(Duration::from_secs(10))
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/121.0.0.0 Safari/537.36")
            .build()
            .map_err(|e| PyErr::new::<PyRuntimeError, _>(e.to_string()))?;

        Ok(FinanceClient { client })
    }

    /// Fetches the home page once to initialize cookies
    fn _initialize_session(&self, py: Python<'_>) -> PyResult<()> {
        // Correct syntax for PyO3 0.23
        py.allow_threads(|| {
            self.client.get("https://www.nseindia.com/")
                .send()
                .map_err(|e| PyErr::new::<PyRuntimeError, _>(e.to_string()))?;
            Ok(())
        })
    }

    fn get_market_status(&self, py: Python<'_>) -> PyResult<String> {
        py.allow_threads(|| {
            // Step 1: Hit home page
            let _ = self.client.get("https://www.nseindia.com/").send();

            // Step 2: API Call
            let response = self.client.get("https://www.nseindia.com/api/marketStatus")
                .header("Referer", "https://www.nseindia.com/")
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