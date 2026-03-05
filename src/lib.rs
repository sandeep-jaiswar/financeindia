use pyo3::prelude::*;
use reqwest::blocking::Client;
use std::collections::HashMap;

#[pyfunction]
fn get_market_status() -> PyResult<String> {
    let client = Client::new();
    let url = "https://www.nseindia.com/api/marketStatus";
    
    // NSE requires a browser-like User-Agent
    let response = client.get(url)
        .header("User-Agent", "Mozilla/5.0")
        .send()
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;

    let body = response.text()
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
    
    Ok(body)
}

/// The Python module named "financeindia"
#[pymodule]
fn financeindia(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(get_market_status, m)?)?;
    Ok(())
}