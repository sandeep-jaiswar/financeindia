use bytes::Bytes;
use chrono::NaiveDate;
use pyo3::IntoPyObjectExt;
use pyo3::exceptions::{PyRuntimeError, PyValueError};
use pyo3::prelude::*;
use reqwest::Client;
use reqwest::header::{ACCEPT, REFERER};
use std::sync::{Arc, RwLock};
use std::time::Duration;
use tokio::time::sleep;

/// Centralized NSE API Headers & URLs
pub const NSE_ALL_REPORTS_URL: &str = "https://www.nseindia.com/all-reports";
pub const NSE_DATE_FMT: &str = "%d-%m-%Y";
pub const SESSION_REFRESH_INTERVAL: Duration = Duration::from_secs(900);
pub const DEFAULT_TIMEOUT: Duration = Duration::from_secs(15);

/// Common helper to build a pre-configured NSE-compatible HTTP Client.
pub fn build_client(extra_headers: Option<reqwest::header::HeaderMap>) -> PyResult<Client> {
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert(
        reqwest::header::USER_AGENT,
        reqwest::header::HeaderValue::from_static(
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36",
        ),
    );
    headers.insert(
        reqwest::header::ACCEPT,
        reqwest::header::HeaderValue::from_static("*/*"),
    );
    headers.insert(
        reqwest::header::ACCEPT_LANGUAGE,
        reqwest::header::HeaderValue::from_static("en-US,en;q=0.9"),
    );
    headers.insert(
        reqwest::header::CACHE_CONTROL,
        reqwest::header::HeaderValue::from_static("no-cache"),
    );
    headers.insert(
        reqwest::header::PRAGMA,
        reqwest::header::HeaderValue::from_static("no-cache"),
    );

    if let Some(extra) = extra_headers {
        for (key, value) in extra.iter() {
            headers.insert(key.clone(), value.clone());
        }
    }

    reqwest::ClientBuilder::new()
        .default_headers(headers)
        .cookie_store(true)
        .timeout(DEFAULT_TIMEOUT)
        .build()
        .map_err(|e| PyErr::new::<PyRuntimeError, _>(e.to_string()))
}

/// Internal helper to parse dates from various common formats.
pub fn parse_date_robust(date: &str) -> PyResult<NaiveDate> {
    let formats = [
        NSE_DATE_FMT,
        "%Y-%m-%d",
        "%d%m%Y",
        "%Y%m%d",
        "%d-%b-%Y",
        "%d%b%Y",
    ];

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
pub async fn fetch_bytes(client: &Client, url: &str, referer: Option<&str>) -> PyResult<Bytes> {
    let mut last_error = String::new();
    let mut delay = Duration::from_millis(500);

    for attempt in 1..=3 {
        let mut rb = client.get(url).header(ACCEPT, "*/*");
        if let Some(r) = referer {
            rb = rb.header(REFERER, r);
        }

        match rb.send().await {
            Ok(resp) => match resp.error_for_status() {
                Ok(checked) => {
                    if let Some(len) = checked.content_length() {
                        if len > 50 * 1024 * 1024 {
                            return Err(PyErr::new::<PyRuntimeError, _>(format!(
                                "Response from {} exceeded 50MB limit",
                                url
                            )));
                        }
                    }
                    match checked.bytes().await {
                        Ok(b) => return Ok(b),
                        Err(e) => {
                            last_error = format!("Error reading body from {} on attempt {}: {}", url, attempt, e);
                            sleep(delay).await;
                            delay *= 2;
                        }
                    }
                }
                Err(e) => {
                    let status = e
                        .status()
                        .map(|s| s.to_string())
                        .unwrap_or_else(|| "Unknown".to_string());
                    last_error =
                        format!("HTTP error {} for {} on attempt {}", status, url, attempt);
                    if e.status() == Some(reqwest::StatusCode::TOO_MANY_REQUESTS)
                        || e.status().map(|s| s.is_server_error()).unwrap_or(false)
                    {
                        sleep(delay).await;
                        delay *= 2;
                    } else {
                        return Err(PyErr::new::<PyRuntimeError, _>(last_error));
                    }
                }
            },
            Ok(resp) => {
                // This branch shouldn't really be reached after error_for_status but for safety:
                last_error = format!("Unknown error for {} on attempt {}", url, attempt);
                sleep(delay).await;
                delay *= 2;
            }
            Err(e) => {
                last_error = format!("Network error for {} on attempt {}: {}", url, attempt, e);
                sleep(delay).await;
                delay *= 2;
            }
        }
    }

    Err(PyErr::new::<PyRuntimeError, _>(format!(
        "Failed to fetch data from {} after 3 attempts. {}",
        url, last_error
    )))
}

/// Shared helper to parse raw JSON bytes into a `serde_json::Value`, mapping errors to Python exceptions.
pub fn parse_json_value(bytes: &[u8]) -> PyResult<serde_json::Value> {
    serde_json::from_slice(bytes)
        .map_err(|e| PyErr::new::<PyRuntimeError, _>(format!("JSON parse error: {e}")))
}

/// Helper to parse CSV string into a Columnar Python dictionary (Dict[str, List[Any]]).
pub fn parse_csv_to_py(py: Python<'_>, csv_bytes: &[u8]) -> PyResult<PyObject> {
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(true)
        .flexible(true)
        .from_reader(csv_bytes);

    let headers = reader
        .headers()
        .map_err(|e| PyErr::new::<PyRuntimeError, _>(format!("CSV Header Error: {}", e)))?
        .clone();

    // Prepare a vector of Python lists for each column
    let mut columns: Vec<pyo3::Bound<'_, pyo3::types::PyList>> = Vec::with_capacity(headers.len());
    for _ in 0..headers.len() {
        columns.push(pyo3::types::PyList::empty(py));
    }

    // Populate columns row by row
    for result in reader.records() {
        let record = result
            .map_err(|e| PyErr::new::<PyRuntimeError, _>(format!("CSV Record Error: {}", e)))?;

        for (i, field) in record.iter().enumerate() {
            if i < columns.len() {
                columns[i].append(field)?;
            }
        }
    }

    // Bind the lists to a single dictionary with the header keys
    let dict = pyo3::types::PyDict::new(py);
    for (i, header) in headers.iter().enumerate() {
        dict.set_item(header, &columns[i])?;
    }

    Ok(dict.into_any().unbind())
}

/// Shared helper to extract the first non-directory file from a ZIP archive as raw Bytes.
pub fn read_first_text_file_from_zip(bytes: bytes::Bytes) -> PyResult<Bytes> {
    let reader = std::io::Cursor::new(bytes);
    let mut archive = zip::ZipArchive::new(reader).map_err(|e| {
        PyErr::new::<PyRuntimeError, _>(format!("Failed to open zip archive: {}", e))
    })?;

    if archive.len() == 0 {
        return Err(PyErr::new::<PyRuntimeError, _>("Zip archive is empty"));
    }

    // Find the first file that is not a directory
    for i in 0..archive.len() {
        let mut file = archive.by_index(i).map_err(|e| {
            PyErr::new::<PyRuntimeError, _>(format!(
                "Failed to get file from zip index {}: {}",
                i, e
            ))
        })?;

        if !file.is_dir() {
            let mut buf = Vec::new();
            use std::io::Read;
            file.read_to_end(&mut buf).map_err(|e| {
                PyErr::new::<PyRuntimeError, _>(format!(
                    "Failed to read zip entry {}: {}",
                    file.name(),
                    e
                ))
            })?;
            return Ok(Bytes::from(buf));
        }
    }

    Err(PyErr::new::<PyRuntimeError, _>(
        "No valid files found in ZIP archive",
    ))
}

/// Helper to parse JSON string into a specific typed Python object.
pub fn parse_json_to_py_typed<'py, T>(py: Python<'py>, json_bytes: &[u8]) -> PyResult<PyObject>
where
    T: for<'de> serde::Deserialize<'de> + IntoPyObject<'py>,
{
    let value: T = serde_json::from_slice(json_bytes)
        .map_err(|e| PyErr::new::<PyRuntimeError, _>(format!("JSON parse error: {}", e)))?;
    value.into_bound_py_any(py).map(|b| b.unbind())
}

/// Helper to parse CSV string into a specific typed Python list.
pub fn parse_csv_to_py_typed<'py, T>(py: Python<'py>, csv_bytes: &[u8]) -> PyResult<PyObject>
where
    T: for<'de> serde::Deserialize<'de> + IntoPyObject<'py>,
{
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(true)
        .flexible(true)
        .from_reader(csv_bytes);

    let list = pyo3::types::PyList::empty(py);
    for result in reader.deserialize() {
        let record: T = result.map_err(|e| {
            PyErr::new::<PyRuntimeError, _>(format!("CSV Deserialize Error: {}", e))
        })?;
        list.append(record.into_bound_py_any(py)?)?;
    }

    Ok(list.into_any().unbind())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_date_robust() {
        assert!(parse_date_robust("2023-01-01").is_ok());
        assert!(parse_date_robust("01-01-2023").is_ok());
        assert!(parse_date_robust("01-Jan-2023").is_ok());
        assert!(parse_date_robust("20230101").is_ok());
        assert!(parse_date_robust("invalid").is_err());
    }

    #[test]
    fn test_date_robust_formats() {
        let d1 = parse_date_robust("2023-05-15").unwrap();
        assert_eq!(d1.to_string(), "2023-05-15");

        let d2 = parse_date_robust("15-05-2023").unwrap();
        assert_eq!(d2.to_string(), "2023-05-15");

        let d3 = parse_date_robust("15-May-2023").unwrap();
        assert_eq!(d3.to_string(), "2023-05-15");
    }
}
