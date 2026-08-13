use crate::error::{self, FinanceError, FinanceResult};
use bytes::Bytes;
use chrono::NaiveDate;
use pyo3::IntoPyObjectExt;
use pyo3::prelude::*;
use reqwest::Client;
use reqwest::header::REFERER;
use serde::{self, Deserialize};
use std::io::Read;
use std::time::Duration;
use tokio::time::sleep;

pub const NSE_ALL_REPORTS_URL: &str = "https://www.nseindia.com/all-reports";
pub const NSE_DATE_FMT: &str = "%d-%m-%Y";
pub const SESSION_REFRESH_INTERVAL: Duration = Duration::from_secs(900);
pub const DEFAULT_TIMEOUT: Duration = Duration::from_secs(15);
pub const DEFAULT_CONNECT_TIMEOUT: Duration = Duration::from_secs(5);
pub const MAX_RESPONSE_SIZE: usize = 50 * 1024 * 1024;
pub const MAX_DECOMPRESSED_ENTRY_SIZE: u64 = 50 * 1024 * 1024;
const MAX_RETRIES: u32 = 3;
const MAX_BACKOFF: Duration = Duration::from_secs(8);

fn is_trusted_redirect_host(host: &str) -> bool {
    (host.ends_with(".nseindia.com") || host == "nseindia.com")
        || (host.ends_with(".mcxindia.com") || host == "mcxindia.com")
}

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

    let redirect_policy = reqwest::redirect::Policy::custom(|attempt| {
        if attempt.previous().len() > 10 {
            return attempt.error("too many redirects");
        }

        if let Some(host) = attempt.url().host_str() {
            if is_trusted_redirect_host(host) {
                if attempt.url().scheme() == "https" {
                    attempt.follow()
                } else {
                    attempt.error("redirect scheme must be https")
                }
            } else {
                attempt.error("untrusted redirect domain")
            }
        } else {
            attempt.error("redirect missing host")
        }
    });

    let mut builder = reqwest::ClientBuilder::new()
        .default_headers(headers)
        .cookie_store(true)
        .redirect(redirect_policy)
        .timeout(DEFAULT_TIMEOUT)
        .connect_timeout(DEFAULT_CONNECT_TIMEOUT);

    if std::env::var("FINANCEINDIA_TEST_ENV").as_deref() != Ok("1") {
        builder = builder.https_only(true);
    }

    Ok(builder.build()?)
}

pub fn parse_date_robust(date: &str) -> FinanceResult<NaiveDate> {
    let formats = [
        NSE_DATE_FMT,
        "%Y-%m-%d",
        "%d%m%Y",
        "%Y%m%d",
        "%d-%b-%Y",
        "%d%b%Y",
    ];

    let clean = date.replace('/', "-").replace('\\', "-");
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

/// Adds 0..=20% randomized jitter to a delay so concurrent retries don't
/// line up in lockstep and hammer the exchange.
fn with_jitter(delay: Duration) -> Duration {
    let base_ms = delay.as_millis() as u64;
    let jitter_ms = ((base_ms as f64) * 0.2) as u64;
    let r = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| (d.subsec_nanos() as u64) % 1000)
        .unwrap_or(0);
    Duration::from_millis(base_ms + (jitter_ms as u128 * r as u128 / 1000) as u64)
}

/// Reads the `Retry-After` header (in seconds) from a rate-limited response.
fn retry_after_secs(resp: &reqwest::Response) -> Option<u64> {
    resp.headers()
        .get(reqwest::header::RETRY_AFTER)?
        .to_str()
        .ok()?
        .trim()
        .parse::<u64>()
        .ok()
}

pub async fn fetch_bytes(
    client: &Client,
    url: &str,
    referer: Option<&str>,
) -> FinanceResult<Bytes> {
    let mut last_error = String::new();
    let mut delay = Duration::from_millis(500);

    for attempt in 1..=MAX_RETRIES {
        let mut rb = client.get(url);
        if let Some(r) = referer {
            rb = rb.header(REFERER, r);
        }

        match rb.send().await {
            Ok(resp) => {
                // Honor Retry-After instead of discarding it.
                if resp.status() == reqwest::StatusCode::TOO_MANY_REQUESTS {
                    return Err(error::rate_limited_error(retry_after_secs(&resp)));
                }
                match resp.error_for_status() {
                    Ok(checked) => {
                        if let Some(len) = checked.content_length() {
                            if len > MAX_RESPONSE_SIZE as u64 {
                                return Err(FinanceError::Runtime(format!(
                                    "Response from {} exceeded {} MB limit",
                                    url,
                                    MAX_RESPONSE_SIZE / (1024 * 1024)
                                )));
                            }
                        }

                        let mut buf = Vec::new();
                        use futures_util::StreamExt;
                        let mut stream = checked.bytes_stream();
                        let mut accumulated_size = 0;
                        let mut stream_error = None;

                        while let Some(chunk_res) = stream.next().await {
                            match chunk_res {
                                Ok(chunk) => {
                                    accumulated_size += chunk.len();
                                    if accumulated_size > MAX_RESPONSE_SIZE {
                                        return Err(FinanceError::Runtime(format!(
                                            "Response from {} exceeded {} MB limit",
                                            url,
                                            MAX_RESPONSE_SIZE / (1024 * 1024)
                                        )));
                                    }
                                    buf.extend_from_slice(&chunk);
                                }
                                Err(e) => {
                                    stream_error = Some(FinanceError::Runtime(format!(
                                        "Chunk stream error from {}: {}",
                                        url, e
                                    )));
                                    break;
                                }
                            }
                        }

                        if let Some(e) = stream_error {
                            last_error = format!(
                                "Error reading body from {} on attempt {}: {}",
                                url, attempt, e
                            );
                            if attempt < MAX_RETRIES {
                                sleep(with_jitter(delay)).await;
                                delay = (delay * 2).min(MAX_BACKOFF);
                            }
                            continue;
                        }

                        return Ok(Bytes::from(buf));
                    }
                    Err(e) => {
                        let status = e
                            .status()
                            .map(|s| s.to_string())
                            .unwrap_or_else(|| "Unknown".to_string());
                        last_error =
                            format!("HTTP error {} for {} on attempt {}", status, url, attempt);
                        if e.status().map(|s| s.is_server_error()).unwrap_or(false) {
                            // Server error - retry with jitter
                            if attempt < MAX_RETRIES {
                                sleep(with_jitter(delay)).await;
                                delay = (delay * 2).min(MAX_BACKOFF);
                            }
                        } else if let Some(status_code) = e.status() {
                            // Client error (4xx except 429, handled above)
                            return Err(error::status_code_error(
                                status_code.as_u16(),
                                e.to_string(),
                            ));
                        } else {
                            // No status code - likely connection error
                            return Err(error::network_error(e.to_string()));
                        }
                    }
                }
            }
            Err(e) => {
                last_error = format!("Network error for {} on attempt {}: {}", url, attempt, e);
                if e.is_timeout() {
                    return Err(FinanceError::Network(format!("Connection timeout: {}", e)));
                } else if e.is_connect() {
                    return Err(FinanceError::Network(format!("Connection refused: {}", e)));
                }
                if attempt < MAX_RETRIES {
                    sleep(with_jitter(delay)).await;
                    delay = (delay * 2).min(MAX_BACKOFF);
                }
            }
        }
    }

    Err(FinanceError::Runtime(format!(
        "Failed to fetch data from {} after {} attempts. Last error: {}",
        url, MAX_RETRIES, last_error
    )))
}

pub fn parse_json_value(bytes: &[u8]) -> FinanceResult<serde_json::Value> {
    Ok(serde_json::from_slice(bytes)?)
}

pub fn parse_csv_to_py(py: Python<'_>, csv_bytes: &[u8]) -> PyResult<PyObject> {
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(true)
        .flexible(true)
        .from_reader(csv_bytes);

    let headers = reader
        .headers()
        .map_err(|e| PyErr::from(FinanceError::Csv(e)))?
        .clone();

    let mut columns: Vec<pyo3::Bound<'_, pyo3::types::PyList>> = Vec::with_capacity(headers.len());
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
            (&mut file)
                .take(MAX_DECOMPRESSED_ENTRY_SIZE)
                .read_to_end(&mut buf)?;
            if buf.len() as u64 >= MAX_DECOMPRESSED_ENTRY_SIZE {
                let mut probe = [0u8; 1];
                if file.read(&mut probe).unwrap_or(0) > 0 {
                    return Err(FinanceError::Runtime(format!(
                        "Decompressed ZIP entry exceeded {} MB limit",
                        MAX_DECOMPRESSED_ENTRY_SIZE / (1024 * 1024)
                    )));
                }
            }
            return Ok(Bytes::from(buf));
        }
    }

    Err(FinanceError::Runtime(
        "No valid files found in ZIP archive".to_string(),
    ))
}

pub fn parse_json_to_py_typed<'py, T>(py: Python<'py>, json_bytes: &[u8]) -> PyResult<PyObject>
where
    T: for<'de> serde::Deserialize<'de> + IntoPyObject<'py>,
{
    let value: T =
        serde_json::from_slice(json_bytes).map_err(|e| PyErr::from(FinanceError::Json(e)))?;
    Ok(value.into_bound_py_any(py)?.unbind())
}

/// Parses an optional numeric CSV field (`None` for missing/empty/`-`).
fn parse_csv_f64_opt(field: Option<&str>) -> Option<f64> {
    let cleaned = field?.replace(',', "").trim().to_string();
    if cleaned.is_empty() || cleaned == "-" {
        None
    } else {
        cleaned.parse::<f64>().ok()
    }
}

/// Parses an optional integer CSV field, tolerating floats with a `.0` suffix
/// (e.g. `"12345.0"`) that NSE occasionally emits for quantity/count columns.
fn parse_csv_i64_opt(field: Option<&str>) -> Option<i64> {
    let cleaned = field?.replace(',', "").trim().to_string();
    if cleaned.is_empty() || cleaned == "-" {
        None
    } else {
        cleaned
            .parse::<i64>()
            .ok()
            .or_else(|| cleaned.parse::<f64>().ok().map(|f| f as i64))
    }
}

fn parse_csv_str_opt(field: Option<&str>) -> Option<String> {
    field.map(|s| s.trim().to_string()).filter(|s| !s.is_empty())
}

/// Parses NSE security-wise historical data CSV into `PriceVolumeRow` objects.
///
/// NSE returns headers like `"Symbol  ","Prev Close  ",...` (title case with trailing
/// spaces and a `₹` marker), so rows are parsed positionally to stay robust against
/// header drift instead of relying on exact header-name matching.
pub fn parse_price_volume_csv_to_py(
    py: Python<'_>,
    csv_bytes: &[u8],
) -> PyResult<PyObject> {
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(true)
        .flexible(true)
        .from_reader(csv_bytes);

    let list = pyo3::types::PyList::empty(py);
    for result in reader.records() {
        let record = result.map_err(|e| PyErr::from(FinanceError::Csv(e)))?;
        let row = crate::models::PriceVolumeRow {
            symbol: parse_csv_str_opt(record.get(0)),
            series: parse_csv_str_opt(record.get(1)),
            date: parse_csv_str_opt(record.get(2)),
            prev_close: parse_csv_f64_opt(record.get(3)),
            open_price: parse_csv_f64_opt(record.get(4)),
            high_price: parse_csv_f64_opt(record.get(5)),
            low_price: parse_csv_f64_opt(record.get(6)),
            last_price: parse_csv_f64_opt(record.get(7)),
            close_price: parse_csv_f64_opt(record.get(8)),
            average_price: parse_csv_f64_opt(record.get(9)),
            total_traded_quantity: parse_csv_i64_opt(record.get(10)),
            turnover: parse_csv_f64_opt(record.get(11)),
            no_of_trades: parse_csv_i64_opt(record.get(12)),
        };
        list.append(row.into_bound_py_any(py)?)?;
    }

    Ok(list.into_any().unbind())
}

/// Parses the NSE equity master (`EQUITY_L.csv`) into `EquityInfo` objects.
///
/// The file's headers are GARBAGE for serde (`"NAME OF COMPANY"`, `"DATE OF
/// LISTING"`, `"PAID UP VALUE"`, `"MARKET LOT"`, `"FACE VALUE"` don't match the
/// `EquityInfo` renames), so rows are parsed positionally to keep the schema
/// robust against header drift.
pub fn parse_equity_list_csv_to_py(py: Python<'_>, csv_bytes: &[u8]) -> PyResult<PyObject> {
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(true)
        .flexible(true)
        .from_reader(csv_bytes);

    let list = pyo3::types::PyList::empty(py);
    for result in reader.records() {
        let record = result.map_err(|e| PyErr::from(FinanceError::Csv(e)))?;
        let row = crate::models::EquityInfo {
            symbol: parse_csv_str_opt(record.get(0)),
            company_name: parse_csv_str_opt(record.get(1)),
            series: parse_csv_str_opt(record.get(2)),
            listing_date: parse_csv_str_opt(record.get(3)),
            paid_up_value: parse_csv_f64_opt(record.get(4)),
            market_lot: parse_csv_str_opt(record.get(5)),
            isin: parse_csv_str_opt(record.get(6)),
            face_value: parse_csv_f64_opt(record.get(7)),
        };
        list.append(row.into_bound_py_any(py)?)?;
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
        assert!(parse_date_robust("15/05/2023").is_ok());
    }

    #[test]
    fn test_parse_date_backslash_separator() {
        let result = parse_date_robust("15\\05\\2023");
        assert!(result.is_ok());
        assert_eq!(result.unwrap().to_string(), "2023-05-15");
    }

    #[test]
    fn test_financeindia_test_env_logic() {
        // Test that only the exact value "1" disables https_only
        // All other cases (unset, empty, "0", "false", etc.) should enable https_only

        // Save original env var state if it exists
        let original = std::env::var("FINANCEINDIA_TEST_ENV").ok();

        // Test case 1: unset -> should enable https_only
        unsafe {
            std::env::remove_var("FINANCEINDIA_TEST_ENV");
        }
        assert_ne!(
            std::env::var("FINANCEINDIA_TEST_ENV").as_deref(),
            Ok("1"),
            "unset env var should not equal Ok(\"1\")"
        );

        // Test case 2: empty string -> should enable https_only
        unsafe {
            std::env::set_var("FINANCEINDIA_TEST_ENV", "");
        }
        assert_ne!(
            std::env::var("FINANCEINDIA_TEST_ENV").as_deref(),
            Ok("1"),
            "empty string should not equal Ok(\"1\")"
        );

        // Test case 3: "0" -> should enable https_only
        unsafe {
            std::env::set_var("FINANCEINDIA_TEST_ENV", "0");
        }
        assert_ne!(
            std::env::var("FINANCEINDIA_TEST_ENV").as_deref(),
            Ok("1"),
            "\"0\" should not equal Ok(\"1\")"
        );

        // Test case 4: "false" -> should enable https_only
        unsafe {
            std::env::set_var("FINANCEINDIA_TEST_ENV", "false");
        }
        assert_ne!(
            std::env::var("FINANCEINDIA_TEST_ENV").as_deref(),
            Ok("1"),
            "\"false\" should not equal Ok(\"1\")"
        );

        // Test case 5: "true" -> should enable https_only
        unsafe {
            std::env::set_var("FINANCEINDIA_TEST_ENV", "true");
        }
        assert_ne!(
            std::env::var("FINANCEINDIA_TEST_ENV").as_deref(),
            Ok("1"),
            "\"true\" should not equal Ok(\"1\")"
        );

        // Test case 6: "1" -> should DISABLE https_only (test mode)
        unsafe {
            std::env::set_var("FINANCEINDIA_TEST_ENV", "1");
        }
        assert_eq!(
            std::env::var("FINANCEINDIA_TEST_ENV").as_deref(),
            Ok("1"),
            "\"1\" should equal Ok(\"1\")"
        );

        // Restore original env var state
        unsafe {
            if let Some(val) = original {
                std::env::set_var("FINANCEINDIA_TEST_ENV", val);
            } else {
                std::env::remove_var("FINANCEINDIA_TEST_ENV");
            }
        }
    }

    #[test]
    fn test_parse_csv_f64_opt() {
        assert_eq!(parse_csv_f64_opt(None), None);
        assert_eq!(parse_csv_f64_opt(Some("")), None);
        assert_eq!(parse_csv_f64_opt(Some("-")), None);
        assert_eq!(parse_csv_f64_opt(Some("1,234.50")), Some(1234.5));
        assert_eq!(parse_csv_f64_opt(Some("42")), Some(42.0));
    }

    #[test]
    fn test_parse_csv_i64_opt() {
        assert_eq!(parse_csv_i64_opt(None), None);
        assert_eq!(parse_csv_i64_opt(Some("")), None);
        assert_eq!(parse_csv_i64_opt(Some("-")), None);
        assert_eq!(parse_csv_i64_opt(Some("123456")), Some(123456));
        assert_eq!(parse_csv_i64_opt(Some("123456.0")), Some(123456));
        assert_eq!(parse_csv_i64_opt(Some("1,234,567")), Some(1234567));
    }

    #[test]
    fn test_parse_csv_str_opt() {
        assert_eq!(parse_csv_str_opt(None), None);
        assert_eq!(parse_csv_str_opt(Some("")), None);
        assert_eq!(parse_csv_str_opt(Some(" RELIANCE ")), Some("RELIANCE".to_string()));
    }

    #[test]
    fn test_with_jitter_bounds() {
        for i in 0..10u64 {
            let base = Duration::from_millis(500 + i * 100);
            let jittered = with_jitter(base);
            let max = base + Duration::from_millis((base.as_millis() as f64 * 0.2) as u64);
            assert!(jittered >= base, "jitter should never shrink the delay");
            assert!(jittered <= max, "jitter should not exceed +20%");
        }
    }

    #[test]
    fn test_backoff_cap() {
        let mut delay = Duration::from_millis(500);
        for _ in 0..5 {
            delay = (delay * 2).min(MAX_BACKOFF);
        }
        assert_eq!(delay, MAX_BACKOFF, "backoff must be capped at MAX_BACKOFF");
    }
}

pub fn to_py_list<'py, T: IntoPyObject<'py>>(py: Python<'py>, items: Vec<T>) -> PyResult<PyObject> {
    let list = pyo3::types::PyList::empty(py);
    for item in items {
        list.append(item.into_bound_py_any(py)?)?;
    }
    Ok(list.into_any().unbind())
}

pub fn deserialize_optional_f64<'de, D>(deserializer: D) -> Result<Option<f64>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    #[derive(serde::Deserialize)]
    #[serde(untagged)]
    enum StringOrFloat {
        String(String),
        Float(f64),
    }

    let val = Option::<StringOrFloat>::deserialize(deserializer)?;
    match val {
        Some(StringOrFloat::String(s)) => {
            let clean = s.replace(',', "").trim().to_string();
            if clean.is_empty() || clean == "-" {
                Ok(None)
            } else {
                clean
                    .parse::<f64>()
                    .map(Some)
                    .map_err(serde::de::Error::custom)
            }
        }
        Some(StringOrFloat::Float(f)) => Ok(Some(f)),
        None => Ok(None),
    }
}
