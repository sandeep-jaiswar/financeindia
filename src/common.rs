use crate::error::*;
use bytes::Bytes;
use chrono::NaiveDate;
use pyo3::IntoPyObjectExt;
use pyo3::prelude::*;
use reqwest::Client;
use reqwest::header::REFERER;
use std::io::Read;
use std::time::Duration;
use tokio::time::sleep;

/// Centralized NSE API constants.
pub const NSE_ALL_REPORTS_URL: &str = "https://www.nseindia.com/all-reports";
pub const NSE_DATE_FMT: &str = "%d-%m-%Y";
pub const SESSION_REFRESH_INTERVAL: Duration = Duration::from_secs(900);
pub const DEFAULT_TIMEOUT: Duration = Duration::from_secs(15);
const MAX_RETRIES: u32 = 3;

/// Common helper to build a pre-configured NSE-compatible HTTP Client.
pub fn build_client(extra_headers: Option<reqwest::header::HeaderMap>) -> FinanceResult<Client> {
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
        headers.extend(extra);
    }

    Ok(reqwest::ClientBuilder::new()
        .default_headers(headers)
        .cookie_store(true)
        .timeout(DEFAULT_TIMEOUT)
        .build()?)
}

/// Internal helper to parse dates from various common formats.
pub fn parse_date_robust(date: &str) -> FinanceResult<NaiveDate> {
    let formats = [
        NSE_DATE_FMT,
        "%Y-%m-%d",
        "%d%m%Y",
        "%Y%m%d",
        "%d-%b-%Y",
        "%d%b%Y",
    ];

    // Normalise slashes to hyphens, then try each known format.
    let clean = date.replace('/', "-");
    for fmt in formats {
        if let Ok(d) = NaiveDate::parse_from_str(&clean, fmt) {
            return Ok(d);
        }
    }

    Err(FinanceError::Runtime(format!(
        "Invalid date format: '{}'. Supported formats include DD-MM-YYYY, YYYY-MM-DD, DD-Mon-YYYY.",
        date
    )))
}

/// Internal helper to execute a GET request with exponential-backoff retries.
///
/// The `ACCEPT: */*` header is already set on every client via `build_client`; no
/// per-request duplicate is emitted. A `Referer` header is added when provided.
pub async fn fetch_bytes(client: &Client, url: &str, referer: Option<&str>) -> FinanceResult<Bytes> {
    let mut last_error = String::new();
    let mut delay = Duration::from_millis(500);

    for attempt in 1..=MAX_RETRIES {
        let mut rb = client.get(url);
        if let Some(r) = referer {
            rb = rb.header(REFERER, r);
        }

        match rb.send().await {
            Ok(resp) => match resp.error_for_status() {
                Ok(checked) => {
                    if let Some(len) = checked.content_length() {
                        if len > 50 * 1024 * 1024 {
                            return Err(FinanceError::Runtime(format!(
                                "Response from {} exceeded 50 MB limit",
                                url
                            )));
                        }
                    }
                    match checked.bytes().await {
                        Ok(b) => return Ok(b),
                        Err(e) => {
                            last_error = format!(
                                "Error reading body from {} on attempt {}: {}",
                                url, attempt, e
                            );
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
                        // Non-retryable HTTP error (e.g. 404, 403).
                        return Err(FinanceError::Http(e));
                    }
                }
            },
            Err(e) => {
                last_error = format!("Network error for {} on attempt {}: {}", url, attempt, e);
                sleep(delay).await;
                delay *= 2;
            }
        }
    }

    Err(FinanceError::Runtime(format!(
        "Failed to fetch data from {} after {} attempts. Last error: {}",
        url, MAX_RETRIES, last_error
    )))
}

/// Parse raw JSON bytes into a `serde_json::Value`.
pub fn parse_json_value(bytes: &[u8]) -> FinanceResult<serde_json::Value> {
    Ok(serde_json::from_slice(bytes)?)
}

/// Parse CSV bytes into a columnar Python dictionary `Dict[str, List[Any]]`.
pub fn parse_csv_to_py(py: Python<'_>, csv_bytes: &[u8]) -> PyResult<PyObject> {
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(true)
        .flexible(true)
        .from_reader(csv_bytes);

    let headers = reader
        .headers()
        .map_err(|e| PyErr::from(FinanceError::Csv(e)))?
        .clone();

    let mut columns: Vec<pyo3::Bound<'_, pyo3::types::PyList>> =
        Vec::with_capacity(headers.len());
    for _ in 0..headers.len() {
        columns.push(pyo3::types::PyList::empty(py));
    }

    for result in reader.records() {
        let record = result.map_err(|e| PyErr::from(FinanceError::Csv(e)))?;
        for (i, field) in record.iter().enumerate() {
            if i < columns.len() {
                columns[i].append(field)?;
            }
        }
    }

    let dict = pyo3::types::PyDict::new(py);
    for (i, header) in headers.iter().enumerate() {
        dict.set_item(header, &columns[i])?;
    }

    Ok(dict.into_any().unbind())
}

/// Extract the first non-directory file from a ZIP archive as raw bytes.
pub fn read_first_text_file_from_zip(bytes: Bytes) -> FinanceResult<Bytes> {
    let reader = std::io::Cursor::new(bytes);
    let mut archive = zip::ZipArchive::new(reader)?;

    if archive.len() == 0 {
        return Err(FinanceError::Runtime("Zip archive is empty".to_string()));
    }

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        if !file.is_dir() {
            let mut buf = Vec::new();
            file.read_to_end(&mut buf)?;
            return Ok(Bytes::from(buf));
        }
    }

    Err(FinanceError::Runtime(
        "No valid files found in ZIP archive".to_string(),
    ))
}

/// Parse JSON bytes into a specific typed Python object.
pub fn parse_json_to_py_typed<'py, T>(py: Python<'py>, json_bytes: &[u8]) -> PyResult<PyObject>
where
    T: for<'de> serde::Deserialize<'de> + IntoPyObject<'py>,
{
    let value: T =
        serde_json::from_slice(json_bytes).map_err(|e| PyErr::from(FinanceError::Json(e)))?;
    Ok(value.into_bound_py_any(py)?.unbind())
}

/// Parse CSV bytes into a specific typed Python list.
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
        let record: T = result.map_err(|e| PyErr::from(FinanceError::Csv(e)))?;
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

    #[test]
    fn test_parse_date_slash_separator() {
        // Slashes should be normalised to hyphens before parsing.
        assert!(parse_date_robust("15/05/2023").is_ok());
    }
}
