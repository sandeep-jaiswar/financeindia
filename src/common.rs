use pyo3::prelude::*;
use pyo3::exceptions::{PyRuntimeError, PyValueError};
use reqwest::blocking::Client;
use reqwest::header::{REFERER, ACCEPT};
use chrono::NaiveDate;
use bytes::Bytes;
use std::time::Duration;
use std::thread;

/// Internal helper to parse dates from various common formats.
pub fn parse_date_robust(date: &str) -> PyResult<NaiveDate> {
    let formats = ["%d-%m-%Y", "%Y-%m-%d", "%d%m%Y", "%Y%m%d", "%d-%b-%Y", "%d%b%Y", "%b-%Y", "%m-%Y"];
    
    let clean = date.replace("/", "-");
    for fmt in formats {
        if let Ok(d) = NaiveDate::parse_from_str(&clean, fmt) {
            return Ok(d);
        }
    }
    
    let raw = clean.replace("-", "");
    if raw.len() == 8 {
        if raw.starts_with("20") || raw.starts_with("19") {
            if let Ok(d) = NaiveDate::parse_from_str(&raw, "%Y%m%d") {
                return Ok(d);
            }
        } else {
            if let Ok(d) = NaiveDate::parse_from_str(&raw, "%d%m%Y") {
                return Ok(d);
            }
        }
    }

    Err(PyErr::new::<PyValueError, _>(format!(
        "Invalid date format: '{}'. Supported formats like DD-MM-YYYY, YYYY-MM-DD are required.", 
        date
    )))
}

/// Internal helper to execute a GET request and return raw bytes.
pub fn fetch_bytes(client: &Client, url: &str, referer: Option<&str>) -> PyResult<Bytes> {
    let mut last_error = String::new();
    let mut delay = Duration::from_millis(500);

    for _attempt in 0..3 {
        let mut rb = client.get(url).header(ACCEPT, "*/*");
        if let Some(r) = referer {
            rb = rb.header(REFERER, r);
        }
        
        match rb.send() {
            Ok(resp) => {
                match resp.error_for_status() {
                    Ok(checked) => return checked.bytes().map_err(|e| PyErr::new::<PyRuntimeError, _>(e.to_string())),
                    Err(e) => {
                        last_error = format!("HTTP error {} for {}", e.status().unwrap_or_default(), url);
                        if e.status() == Some(reqwest::StatusCode::TOO_MANY_REQUESTS) || e.status().map(|s| s.is_server_error()).unwrap_or(false) {
                            thread::sleep(delay);
                            delay *= 2;
                        } else {
                            return Err(PyErr::new::<PyRuntimeError, _>(last_error));
                        }
                    }
                }
            }
            Err(e) => {
                last_error = format!("Network error for {}: {}", url, e);
                thread::sleep(delay);
                delay *= 2;
            }
        }
    }
    
    Err(PyErr::new::<PyRuntimeError, _>(format!("Failed after 3 attempts. Last error: {}", last_error)))
}

/// Internal helper to execute a GET request and return text.
pub fn fetch_text(client: &Client, url: &str, referer: Option<&str>) -> PyResult<String> {
    let bytes = fetch_bytes(client, url, referer)?;
    Ok(String::from_utf8_lossy(&bytes).into_owned())
}

/// Helper to parse CSV string into a Python list of dicts directly.
pub fn parse_csv_to_py(py: Python<'_>, csv_text: &str) -> PyResult<PyObject> {
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(true)
        .flexible(true)
        .from_reader(csv_text.as_bytes());

    let headers = reader.headers()
        .map_err(|e| PyErr::new::<PyRuntimeError, _>(format!("CSV Header Error: {}", e)))?
        .clone();

    let list = pyo3::types::PyList::empty(py);
    for result in reader.records() {
        let record = result.map_err(|e| PyErr::new::<PyRuntimeError, _>(format!("CSV Record Error: {}", e)))?;
        let dict = pyo3::types::PyDict::new(py);
        for (header, field) in headers.iter().zip(record.iter()) {
            dict.set_item(header, field)?;
        }
        list.append(dict)?;
    }

    Ok(list.to_object(py))
}

/// Helper to parse CSV string into a list of dicts (JSON string for now).
pub fn parse_csv_to_json(csv_text: &str) -> PyResult<String> {
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(true)
        .flexible(true)
        .from_reader(csv_text.as_bytes());

    let headers = reader.headers()
        .map_err(|e| PyErr::new::<PyRuntimeError, _>(format!("CSV Header Error: {}", e)))?
        .clone();

    let mut records = Vec::new();
    for result in reader.records() {
        let record = result.map_err(|e| PyErr::new::<PyRuntimeError, _>(format!("CSV Record Error: {}", e)))?;
        let mut map = serde_json::Map::new();
        for (header, field) in headers.iter().zip(record.iter()) {
            map.insert(header.to_string(), serde_json::Value::String(field.to_string()));
        }
        records.push(serde_json::Value::Object(map));
    }

    serde_json::to_string(&records).map_err(|e| PyErr::new::<PyRuntimeError, _>(e.to_string()))
}
